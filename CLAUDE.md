# Omnisearch

CLI multi-source search aggregator (Rust). Queries 50+ APIs in parallel, returns unified JSON.

## Quick reference

```bash
omnisearch "query"                        # all sources, JSON output
omnisearch "query" -c academic,news       # filter by category
omnisearch "query" -s wikipedia,arxiv     # specific sources
omnisearch "query" -f table|markdown      # output format
omnisearch "query" -n 20                  # max results
omnisearch "query" -v                     # verbose (progress on stderr)
omnisearch sources                        # list sources + status
omnisearch config set <source> "<key>"    # add API key
```

## Categories

`general`, `academic`, `news`, `finance`, `weather`, `media`, `code`, `reference`, `security`, `geo`, `music`

## Architecture

- Entry point: `src/main.rs` → dispatches CLI commands
- Source trait: `src/sources/mod.rs` → `build_sources()` registers all 50 sources
- Each source: `src/sources/<name>.rs` → implements `Source` trait with `async fn search()`
- Engine: `src/engine.rs` → parallel query orchestrator with per-source timeout
- Config: `~/.config/omnisearch/config.toml` (macOS: `~/Library/Application Support/omnisearch/`)

## Build & install

```bash
cargo build --release
cargo install --path .
```
