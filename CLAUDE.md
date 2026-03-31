# Omnisearch

CLI multi-source search aggregator (Rust). Queries 56 APIs in parallel, returns unified JSON.

## Quick reference

```bash
omnisearch "query"                        # all sources, JSON output
omnisearch "query" -c academic,news       # filter by category
omnisearch "query" -c market              # market research sources
omnisearch "query" -s wikipedia,arxiv     # specific sources
omnisearch "query" -f table|markdown      # output format
omnisearch "query" -n 20                  # max results
omnisearch "query" -v                     # verbose (progress on stderr)
omnisearch "query" --no-cache             # skip cache, force fresh search
omnisearch sources                        # list sources + status
omnisearch config set <source> "<key>"    # add API key
omnisearch mcp                            # start MCP server (stdin/stdout)
```

## Categories

`general`, `academic`, `news`, `finance`, `weather`, `media`, `code`, `reference`, `security`, `geo`, `music`, `market`

## Architecture

- Entry point: `src/main.rs` → dispatches CLI commands
- Source trait: `src/sources/mod.rs` → `build_sources()` registers all 56 sources
- Each source: `src/sources/<name>.rs` → implements `Source` trait with `async fn search()`
- Engine: `src/engine.rs` → parallel query orchestrator with per-source timeout
- Cache: `src/cache.rs` → disk-based SHA256-keyed cache with TTL (`~/.cache/omnisearch/`)
- MCP server: `src/mcp.rs` → JSON-RPC 2.0 over stdin/stdout, exposes `search` and `list_sources` tools
- Config: `~/.config/omnisearch/config.toml` (macOS: `~/Library/Application Support/omnisearch/`)

## Build & install

```bash
cargo build --release
cargo install --path .
```

## MCP integration

Add to Claude Code `settings.json`:
```json
{
  "mcpServers": {
    "omnisearch": {
      "command": "omnisearch",
      "args": ["mcp"]
    }
  }
}
```
