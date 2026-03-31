use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;
use crate::engine::SearchEngine;
use crate::http::build_client;
use crate::ranking::dedup_and_rank;
use crate::rate_limit::RateLimiterRegistry;
use crate::sources::{all_source_info, build_sources};
use crate::types::{Category, SearchQuery};
use std::io::{self, BufRead, Write};

// ── JSON-RPC 2.0 base types ──

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

impl JsonRpcResponse {
    fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Option<Value>, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}

// ── MCP Tool definitions ──

#[derive(Debug, Serialize)]
struct ToolDef {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

fn tool_definitions() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "search".to_string(),
            description: "Search across 50+ sources (web, academic, news, finance, code, market, etc). Returns unified results.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "categories": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Filter by categories: general, academic, news, finance, weather, media, code, reference, security, geo, music, market"
                    },
                    "sources": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Filter by specific source names (e.g. wikipedia, arxiv, hacker_news, sec_edgar)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 20)"
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDef {
            name: "list_sources".to_string(),
            description: "List all available search sources with their status (enabled, needs key, ready).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}

// ── MCP server loop ──

pub async fn run_mcp_server() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let config = Config::load().unwrap_or_default();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::error(None, -32700, format!("Parse error: {e}"));
                let mut out = stdout.lock();
                let _ = writeln!(out, "{}", serde_json::to_string(&resp).unwrap());
                let _ = out.flush();
                continue;
            }
        };

        // Notifications (no id) don't get a response
        if request.id.is_none() {
            continue;
        }

        let response = handle_request(&request, &config).await;
        let mut out = stdout.lock();
        let _ = writeln!(out, "{}", serde_json::to_string(&response).unwrap());
        let _ = out.flush();
    }
}

async fn handle_request(req: &JsonRpcRequest, config: &Config) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => JsonRpcResponse::success(
            req.id.clone(),
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "omnisearch",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        ),
        "tools/list" => {
            let tools = tool_definitions();
            JsonRpcResponse::success(req.id.clone(), serde_json::json!({ "tools": tools }))
        }
        "tools/call" => handle_tool_call(req, config).await,
        _ => JsonRpcResponse::error(
            req.id.clone(),
            -32601,
            format!("Method not found: {}", req.method),
        ),
    }
}

async fn handle_tool_call(req: &JsonRpcRequest, config: &Config) -> JsonRpcResponse {
    let params = req.params.as_ref().unwrap_or(&Value::Null);
    let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    match tool_name {
        "search" => handle_search(req.id.clone(), &arguments, config).await,
        "list_sources" => handle_list_sources(req.id.clone(), config),
        _ => JsonRpcResponse::error(
            req.id.clone(),
            -32602,
            format!("Unknown tool: {tool_name}"),
        ),
    }
}

async fn handle_search(id: Option<Value>, args: &Value, config: &Config) -> JsonRpcResponse {
    let query_str = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => {
            return JsonRpcResponse::error(id, -32602, "Missing required parameter: query".into());
        }
    };

    let max_results = args
        .get("max_results")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(config.general.max_results);

    let categories: Option<Vec<Category>> = args.get("categories").and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|c| c.as_str()?.parse::<Category>().ok())
                .collect()
        })
    });

    let sources_filter: Option<Vec<String>> = args.get("sources").and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
    });

    let client = build_client(config.general.timeout_secs);
    let sources = build_sources(config);

    let mut rate_limiters = RateLimiterRegistry::new();
    for source in &sources {
        rate_limiters.register(source.name(), source.rate_limit_per_minute());
    }

    let engine = SearchEngine::new(client, sources, rate_limiters, config.general.timeout_secs);
    let search_query = SearchQuery {
        query: query_str,
        max_results,
        categories,
        sources: sources_filter,
    };

    let (results, errors, sources_queried) = engine.search(&search_query, None).await;
    let results = dedup_and_rank(results);
    let results: Vec<_> = results.into_iter().take(max_results).collect();

    let mut text = format!(
        "Found {} results from {} sources.\n\n",
        results.len(),
        sources_queried.len()
    );
    for (i, r) in results.iter().enumerate() {
        text.push_str(&format!(
            "{}. **{}** ({})\n   {}\n   {}\n\n",
            i + 1,
            r.title,
            r.source,
            r.snippet,
            r.url
        ));
    }

    if !errors.is_empty() {
        text.push_str(&format!("\n({} sources had errors)", errors.len()));
    }

    JsonRpcResponse::success(
        id,
        serde_json::json!({
            "content": [{
                "type": "text",
                "text": text
            }]
        }),
    )
}

fn handle_list_sources(id: Option<Value>, config: &Config) -> JsonRpcResponse {
    let infos = all_source_info(config);
    let mut text = String::from("Available sources:\n\n");

    for info in &infos {
        let status = if !info.enabled {
            "disabled"
        } else if info.requires_key && !info.key_configured {
            "needs key"
        } else {
            "ready"
        };
        text.push_str(&format!(
            "- {} [{}] ({})\n",
            info.name, status, info.category
        ));
    }

    JsonRpcResponse::success(
        id,
        serde_json::json!({
            "content": [{
                "type": "text",
                "text": text
            }]
        }),
    )
}
