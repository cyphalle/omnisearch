# Cache Local + Serveur MCP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a disk-based search result cache with TTL, and an MCP server mode so Claude Code (and other agents) can use omnisearch as a tool.

**Architecture:** Cache uses SHA256 hash of query params as key, stores JSON files in `~/.cache/omnisearch/`. MCP server implements the JSON-RPC 2.0 protocol over stdin/stdout, exposing `search` and `list_sources` as tools. Both features are independent modules wired into the existing CLI.

**Tech Stack:** Rust, sha2 (hashing), serde_json (cache serialization), existing tokio/clap/reqwest stack.

---

## File Structure

| File | Responsibility |
|------|---------------|
| `src/cache.rs` (create) | Cache read/write/invalidation logic |
| `src/mcp.rs` (create) | MCP JSON-RPC protocol handler |
| `src/cli.rs` (modify) | Add `--no-cache` flag and `mcp` subcommand |
| `src/main.rs` (modify) | Wire cache into search flow, add mcp command handler |
| `src/lib.rs` (modify) | Register new modules |
| `src/types.rs` (modify) | Add Serialize/Deserialize to SearchResponse |
| `Cargo.toml` (modify) | Add `sha2` dependency |
| `tests/cache_test.rs` (create) | Cache unit tests |
| `tests/mcp_test.rs` (create) | MCP protocol tests |

---

### Task 1: Add `sha2` dependency and register modules

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`

- [ ] **Step 1: Add sha2 to Cargo.toml**

In `Cargo.toml`, add under `[dependencies]`:

```toml
sha2 = "0.10"
```

- [ ] **Step 2: Register cache and mcp modules in lib.rs**

In `src/lib.rs`, add:

```rust
pub mod cache;
pub mod mcp;
```

- [ ] **Step 3: Create empty module files**

Create `src/cache.rs` with:

```rust
// Cache module — disk-based search result cache with TTL
```

Create `src/mcp.rs` with:

```rust
// MCP server module — JSON-RPC 2.0 over stdin/stdout
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/cache.rs src/mcp.rs
git commit -m "chore: add sha2 dep, scaffold cache and mcp modules"
```

---

### Task 2: Implement cache module

**Files:**
- Create: `src/cache.rs`
- Create: `tests/cache_test.rs`

The cache stores search results as JSON files keyed by SHA256 hash of (query + categories + sources). Default TTL = 1 hour. Cache dir = `~/.cache/omnisearch/`.

- [ ] **Step 1: Write the failing test for cache round-trip**

Create `tests/cache_test.rs`:

```rust
use omnisearch::cache::Cache;
use omnisearch::types::{Category, SearchResult, SearchResponse, SourceError};
use std::time::Duration;

#[test]
fn test_cache_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let cache = Cache::new(dir.path().to_path_buf(), Duration::from_secs(3600));

    let response = SearchResponse {
        query: "rust programming".to_string(),
        total_results: 1,
        sources_queried: vec!["wikipedia".to_string()],
        results: vec![SearchResult {
            title: "Rust".to_string(),
            url: "https://en.wikipedia.org/wiki/Rust_(programming_language)".to_string(),
            snippet: "Rust is a language".to_string(),
            source: "wikipedia".to_string(),
            category: Category::Reference,
            published: None,
            author: None,
            relevance_score: 0.95,
        }],
        errors: vec![],
    };

    let categories: Option<Vec<Category>> = None;
    let sources: Option<Vec<String>> = None;

    cache.put("rust programming", &categories, &sources, &response).unwrap();
    let cached = cache.get("rust programming", &categories, &sources).unwrap();

    assert!(cached.is_some());
    let cached = cached.unwrap();
    assert_eq!(cached.query, "rust programming");
    assert_eq!(cached.results.len(), 1);
    assert_eq!(cached.results[0].title, "Rust");
}

#[test]
fn test_cache_miss() {
    let dir = tempfile::tempdir().unwrap();
    let cache = Cache::new(dir.path().to_path_buf(), Duration::from_secs(3600));

    let result = cache.get("nonexistent", &None, &None).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_cache_expired() {
    let dir = tempfile::tempdir().unwrap();
    // TTL of 0 seconds = always expired
    let cache = Cache::new(dir.path().to_path_buf(), Duration::from_secs(0));

    let response = SearchResponse {
        query: "test".to_string(),
        total_results: 0,
        sources_queried: vec![],
        results: vec![],
        errors: vec![],
    };

    cache.put("test", &None, &None, &response).unwrap();

    // Sleep 1ms to ensure expiry
    std::thread::sleep(Duration::from_millis(10));

    let cached = cache.get("test", &None, &None).unwrap();
    assert!(cached.is_none());
}
```

- [ ] **Step 2: Add tempfile dev-dependency**

In `Cargo.toml`, add:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --test cache_test`
Expected: FAIL — `Cache` type doesn't exist yet

- [ ] **Step 4: Make SearchResponse serializable for cache**

In `src/types.rs`, add `Deserialize` to `SearchResponse` and `SourceError`:

Change:
```rust
#[derive(Debug, Serialize)]
pub struct SearchResponse {
```
to:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
```

Change:
```rust
#[derive(Debug, Serialize)]
pub struct SourceError {
```
to:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceError {
```

- [ ] **Step 5: Implement cache module**

Write `src/cache.rs`:

```rust
use crate::error::OmniResult;
use crate::types::{Category, SearchResponse};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub struct Cache {
    dir: PathBuf,
    ttl: Duration,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    timestamp: u64,
    response: SearchResponse,
}

impl Cache {
    pub fn new(dir: PathBuf, ttl: Duration) -> Self {
        Self { dir, ttl }
    }

    pub fn default_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("omnisearch")
    }

    pub fn get(
        &self,
        query: &str,
        categories: &Option<Vec<Category>>,
        sources: &Option<Vec<String>>,
    ) -> OmniResult<Option<SearchResponse>> {
        let path = self.cache_path(query, categories, sources);
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let entry: CacheEntry = match serde_json::from_str(&content) {
            Ok(e) => e,
            Err(_) => {
                // Corrupted cache file, remove it
                let _ = std::fs::remove_file(&path);
                return Ok(None);
            }
        };

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now.saturating_sub(entry.timestamp) > self.ttl.as_secs() {
            let _ = std::fs::remove_file(&path);
            return Ok(None);
        }

        Ok(Some(entry.response))
    }

    pub fn put(
        &self,
        query: &str,
        categories: &Option<Vec<Category>>,
        sources: &Option<Vec<String>>,
        response: &SearchResponse,
    ) -> OmniResult<()> {
        std::fs::create_dir_all(&self.dir)?;

        let entry = CacheEntry {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response: response.clone(),
        };

        let path = self.cache_path(query, categories, sources);
        let content = serde_json::to_string(&entry)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn cache_path(
        &self,
        query: &str,
        categories: &Option<Vec<Category>>,
        sources: &Option<Vec<String>>,
    ) -> PathBuf {
        let key = Self::cache_key(query, categories, sources);
        self.dir.join(format!("{key}.json"))
    }

    fn cache_key(
        query: &str,
        categories: &Option<Vec<Category>>,
        sources: &Option<Vec<String>>,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        if let Some(cats) = categories {
            let mut cat_strs: Vec<String> = cats.iter().map(|c| c.to_string()).collect();
            cat_strs.sort();
            hasher.update(cat_strs.join(",").as_bytes());
        }
        if let Some(srcs) = sources {
            let mut sorted = srcs.clone();
            sorted.sort();
            hasher.update(sorted.join(",").as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test --test cache_test`
Expected: all 3 tests PASS

- [ ] **Step 7: Commit**

```bash
git add src/cache.rs src/types.rs tests/cache_test.rs Cargo.toml
git commit -m "feat: add disk-based search cache with TTL"
```

---

### Task 3: Wire cache into CLI and search flow

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/main.rs`
- Modify: `src/config.rs`

- [ ] **Step 1: Add `--no-cache` flag to CLI**

In `src/cli.rs`, add to the `Cli` struct after the `verbose` field:

```rust
    /// Disable cache, force fresh search
    #[arg(long)]
    pub no_cache: bool,
```

- [ ] **Step 2: Add cache TTL to config**

In `src/config.rs`, add to `GeneralConfig`:

```rust
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,
```

Add the default function:

```rust
fn default_cache_ttl() -> u64 {
    3600
}
```

Update the `Default` impl for `GeneralConfig`:

```rust
impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout(),
            max_results: default_max_results(),
            default_format: default_format(),
            cache_ttl_secs: default_cache_ttl(),
        }
    }
}
```

- [ ] **Step 3: Wire cache into cmd_search in main.rs**

In `src/main.rs`, add import at top:

```rust
use omnisearch::cache::Cache;
use std::time::Duration;
```

In the `cmd_search` function, after `let categories` is computed and before `let client = build_client(...)`, insert cache check:

```rust
    let cache_ttl = Duration::from_secs(config.general.cache_ttl_secs);
    let cache = Cache::new(Cache::default_dir(), cache_ttl);

    // Check cache first (unless --no-cache)
    if !cli.no_cache {
        if let Ok(Some(cached)) = cache.get(&query, &categories, &cli.sources) {
            if verbose {
                eprintln!("{}", "Cache hit — using cached results".green());
            }
            let results: Vec<_> = cached.results.into_iter().take(max_results).collect();
            let response = SearchResponse {
                query: cached.query,
                total_results: results.len(),
                sources_queried: cached.sources_queried,
                results,
                errors: cached.errors,
            };
            println!("{}", format_output(&response, format));
            return;
        }
    }
```

After the existing `format_output` line (the final `println!`), insert cache write just before it:

```rust
    // Cache the response
    if !cli.no_cache {
        let _ = cache.put(&query, &categories, &cli.sources, &response);
    }
```

- [ ] **Step 4: Verify it compiles and runs**

Run: `cargo build`
Expected: compiles

Run: `cargo run -- "test query" --no-cache -n 5 2>/dev/null | head -5`
Expected: JSON output (may have errors for unconfigured sources, that's fine)

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs src/config.rs src/main.rs
git commit -m "feat: wire cache into search flow with --no-cache flag"
```

---

### Task 4: Implement MCP server — protocol types

**Files:**
- Modify: `src/mcp.rs`

The MCP protocol uses JSON-RPC 2.0. We need types for requests/responses and the `initialize`, `tools/list`, `tools/call` methods.

- [ ] **Step 1: Write MCP protocol types**

Replace `src/mcp.rs` with:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── JSON-RPC 2.0 base types ──

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }

    pub fn notification_result(method: &str, params: Value) -> String {
        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        serde_json::to_string(&notif).unwrap()
    }
}

// ── MCP Tool definitions ──

#[derive(Debug, Serialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

pub fn tool_definitions() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "search".to_string(),
            description: "Search across 50+ sources (web, academic, news, finance, code, etc). Returns unified results.".to_string(),
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
                        "description": "Filter by categories: general, academic, news, finance, weather, media, code, reference, security, geo, music"
                    },
                    "sources": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Filter by specific source names (e.g. wikipedia, arxiv, hacker_news)"
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

use crate::config::Config;
use crate::engine::SearchEngine;
use crate::http::build_client;
use crate::rate_limit::RateLimiterRegistry;
use crate::sources::{all_source_info, build_sources};
use crate::types::{Category, SearchQuery};
use crate::ranking::dedup_and_rank;
use std::io::{self, BufRead, Write};

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

        let response = handle_request(&request, &config).await;
        let mut out = stdout.lock();
        let _ = writeln!(out, "{}", serde_json::to_string(&response).unwrap());
        let _ = out.flush();
    }
}

async fn handle_request(req: &JsonRpcRequest, config: &Config) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => {
            JsonRpcResponse::success(
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
            )
        }
        "notifications/initialized" => {
            // Client acknowledgement — no response needed for notifications,
            // but since we're in a request handler, return empty success
            JsonRpcResponse::success(req.id.clone(), serde_json::json!({}))
        }
        "tools/list" => {
            let tools = tool_definitions();
            JsonRpcResponse::success(
                req.id.clone(),
                serde_json::json!({ "tools": tools }),
            )
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
    let params = req.params.as_ref().unwrap_or(&serde_json::Value::Null);
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

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

async fn handle_search(
    id: Option<Value>,
    args: &Value,
    config: &Config,
) -> JsonRpcResponse {
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

    let mut text = format!("Found {} results from {} sources.\n\n", results.len(), sources_queried.len());
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
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/mcp.rs
git commit -m "feat: implement MCP server with search and list_sources tools"
```

---

### Task 5: Wire MCP server into CLI

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add Mcp subcommand to CLI**

In `src/cli.rs`, add a variant to `Commands`:

```rust
    /// Start as MCP server (stdin/stdout JSON-RPC)
    Mcp,
```

- [ ] **Step 2: Add mcp handler in main.rs**

In `src/main.rs`, add import:

```rust
use omnisearch::mcp;
```

In the `match cli.command` block, add before `None =>`:

```rust
        Some(Commands::Mcp) => mcp::run_mcp_server().await,
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: compiles

- [ ] **Step 4: Smoke-test the MCP server**

Run:
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run -- mcp
```
Expected: JSON response with `serverInfo.name = "omnisearch"`

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs src/main.rs
git commit -m "feat: add 'mcp' subcommand to start MCP server"
```

---

### Task 6: Write MCP integration test

**Files:**
- Create: `tests/mcp_test.rs`

- [ ] **Step 1: Write test for MCP tool listing**

Create `tests/mcp_test.rs`:

```rust
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_mcp_initialize_and_list_tools() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "mcp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start omnisearch mcp");

    let stdin = child.stdin.as_mut().unwrap();

    // Send initialize
    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{}}}}"#).unwrap();
    // Send tools/list
    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{{}}}}"#).unwrap();
    // Close stdin to let server exit
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(lines.len() >= 2, "Expected at least 2 response lines, got: {}", stdout);

    // Check initialize response
    let init_resp: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(init_resp["result"]["serverInfo"]["name"], "omnisearch");

    // Check tools/list response
    let tools_resp: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    let tools = tools_resp["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"search"));
    assert!(tool_names.contains(&"list_sources"));
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test --test mcp_test`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add tests/mcp_test.rs
git commit -m "test: add MCP server integration test"
```

---

### Task 7: Final verification and cleanup

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 2: Build release**

Run: `cargo build --release`
Expected: compiles

- [ ] **Step 3: Verify cache works end-to-end**

Run:
```bash
# First search (cache miss)
cargo run -- "rust programming" -s wikipedia -v 2>&1 | head -3
# Second search (cache hit — should be instant)
cargo run -- "rust programming" -s wikipedia -v 2>&1 | head -3
# With --no-cache (fresh)
cargo run -- "rust programming" -s wikipedia -v --no-cache 2>&1 | head -3
```

- [ ] **Step 4: Verify MCP smoke test**

Run:
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | cargo run -- mcp
```
Expected: two JSON lines, second listing `search` and `list_sources` tools.

- [ ] **Step 5: Final commit if any cleanup was needed**

```bash
git add -A
git commit -m "chore: final cleanup for cache + mcp features"
```
