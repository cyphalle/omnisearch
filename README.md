# omnisearch

CLI multi-source search aggregator. Queries 50+ APIs in parallel and returns unified results. Designed for use with Claude Code (JSON output by default).

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Search all configured sources
omnisearch "climate change"

# Filter by category
omnisearch "climate change" -c academic,news

# Filter by specific sources
omnisearch "climate change" -s wikipedia,arxiv,hacker_news

# Output formats: json (default), table, markdown
omnisearch "climate change" -f table

# Limit results
omnisearch "climate change" -n 10

# Custom timeout per source
omnisearch "climate change" -t 15

# Verbose mode (show progress on stderr)
omnisearch "climate change" -v

# List all sources and their status
omnisearch sources

# Manage configuration
omnisearch config init          # create default config
omnisearch config show          # show current config
omnisearch config set brave          # set an API key (read from stdin)
```

## Sources (50)

### No API key required (22) — ready out of the box

| Source | Category | Description |
|--------|----------|-------------|
| Wikipedia | Reference | Encyclopedia articles |
| Wikidata | Reference | Structured knowledge base |
| arXiv | Academic | Preprint papers (physics, math, CS) |
| OpenAlex | Academic | Open scholarly metadata |
| CrossRef | Academic | DOI metadata and citations |
| PubMed | Academic | Biomedical literature |
| DuckDuckGo | General | Instant answers |
| Hacker News | Code | Tech news and discussions |
| Stack Exchange | Code | Programming Q&A |
| GDELT | News | Global news monitoring |
| Internet Archive | General | Digital library |
| DBpedia | Reference | Structured Wikipedia data |
| Datamuse | Reference | Word relationships and definitions |
| MusicBrainz | Music | Music metadata |
| Open-Meteo | Weather | Weather forecasts |
| RestCountries | Geo | Country information |
| GeoNames | Geo | Geographic database |
| Frankfurter | Finance | Exchange rates (ECB) |
| World Bank | Finance | Development data and reports |
| Common Crawl | General | Web crawl index |
| crt.sh | Security | Certificate transparency logs |
| Have I Been Pwned | Security | Data breach search |

### API key required (28) — free tiers available

| Source | Category | Free tier |
|--------|----------|-----------|
| Brave Search | General | 2,000/mo |
| Google Custom Search | General | 100/day |
| GitHub | Code | 5,000/hr |
| Semantic Scholar | Academic | 100/sec |
| NewsAPI | News | 100/day |
| Guardian | News | 500/day |
| NYT | News | 500/day |
| GNews | News | 100/day |
| MediaStack | News | 100/mo |
| Currents | News | 600/day |
| Event Registry | News | 500/day |
| YouTube | Media | 10,000 units/day |
| TMDB | Media | Unlimited |
| OMDb | Media | 1,000/day |
| Unsplash | Media | 50/hr |
| Pexels | Media | 200/hr |
| Alpha Vantage | Finance | 25/day |
| CoinGecko | Finance | 10,000/mo |
| Polygon.io | Finance | 5/min |
| FRED | Finance | Unlimited |
| Open Exchange Rates | Finance | 1,000/mo |
| OpenWeatherMap | Weather | 1,000/day |
| Wolfram Alpha | Reference | 2,000/mo |
| Shodan | Security | 1/sec |
| VirusTotal | Security | 4/min |
| URLScan.io | Security | 60/min |
| ipinfo.io | Security | 50,000/mo |
| Mapbox | Geo | 100,000/mo |

## Configuration

Config file location: `~/.config/omnisearch/config.toml` (macOS: `~/Library/Application Support/omnisearch/config.toml`)

```bash
# Generate default config
omnisearch config init

# Add API keys (prompted on stdin: the key stays out of shell history)
omnisearch config set brave
gh auth token | omnisearch config set github -
omnisearch config set tmdb "abc123..."   # also accepted, but lands in shell history

# Show the config, API keys masked to their last 4 characters
omnisearch config show
```

## Security

- **Key storage.** The config file holds every API key in plain text. It is written with mode `0600` in a `0700` directory. `config show` masks keys.
- **Keys in URLs.** About 20 providers only accept the key as a query parameter (Guardian, NewsAPI, NYT, FRED, Polygon, ...). Keys are URL-encoded, and HTTP errors are stripped of the request URL before they are printed, cached or returned over MCP.
- **Plain HTTP providers.** `mediastack` is called over `http://`: its free plan does not serve HTTPS, so its key travels unencrypted. Disable the source (`[sources.mediastack] enabled = false`) if that matters on your network. `geonames` also uses `http://` with the public `demo` account (no secret).
- **Cache.** Results are cached under the user cache directory (`~/Library/Caches/omnisearch` on macOS) with mode `0600`. Errors are never cached. Delete the directory or pass `--no-cache` for sensitive queries.
- **Prompt injection (MCP).** Titles, snippets and URLs come from third parties and can contain text written to manipulate an LLM. The MCP `search` tool wraps them in an `<untrusted_search_results>` block, flattens them to one line, replaces `<` and `>`, truncates snippets to 500 characters and caps `max_results` at 50. This reduces the risk without removing it: do not give an agent that reads these results tools with irreversible effects without a human check.
- **Response size.** Sources that can return very large payloads (`crtsh`, `common_crawl`) are read with a size cap.

## Architecture

- **Async parallel queries** — all sources queried concurrently via tokio
- **Per-source timeout** (8s default) — slow sources don't block others
- **Rate limiting** — per-source via governor (GCRA algorithm)
- **Partial failure** — one source failing doesn't affect others
- **URL dedup** — results deduplicated by normalized URL
- **Relevance ranking** — results sorted by source-specific relevance scores

## JSON output structure

```json
{
  "query": "search terms",
  "total_results": 15,
  "sources_queried": ["wikipedia", "arxiv", "hacker_news"],
  "results": [
    {
      "title": "Result title",
      "url": "https://...",
      "snippet": "Description text...",
      "source": "wikipedia",
      "category": "reference",
      "published": "2024-01-15T00:00:00Z",
      "author": "Author Name",
      "relevance_score": 0.95
    }
  ],
  "errors": [
    {
      "source": "some_source",
      "error": "Timeout querying some_source"
    }
  ]
}
```

## Categories

`general`, `academic`, `news`, `finance`, `weather`, `media`, `code`, `reference`, `security`, `geo`, `music`
