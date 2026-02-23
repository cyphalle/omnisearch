use clap::Parser;
use colored::Colorize;
use omnisearch::cli::{Cli, Commands, ConfigAction};
use omnisearch::config::Config;
use omnisearch::engine::SearchEngine;
use omnisearch::http::build_client;
use omnisearch::output::format_output;
use omnisearch::ranking::dedup_and_rank;
use omnisearch::rate_limit::RateLimiterRegistry;
use omnisearch::sources::{all_source_info, build_sources};
use omnisearch::types::{Category, SearchQuery, SearchResponse};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("{}: {}", "Config error".red(), e);
        Config::default()
    });

    match cli.command {
        Some(Commands::Sources) => cmd_sources(&config),
        Some(Commands::Config { action }) => cmd_config(action, config),
        None => {
            if let Some(ref query) = cli.query {
                cmd_search(query.clone(), &cli, config).await;
            } else {
                eprintln!("{}", "No query provided. Use --help for usage.".red());
                std::process::exit(1);
            }
        }
    }
}

fn cmd_sources(config: &Config) {
    let infos = all_source_info(config);

    println!("{}", "Available Sources:".bold());
    println!();

    let mut current_category = None;
    let mut sorted = infos;
    sorted.sort_by(|a, b| {
        a.category
            .to_string()
            .cmp(&b.category.to_string())
            .then(a.name.cmp(&b.name))
    });

    for info in &sorted {
        let cat_str = info.category.to_string();
        if current_category.as_ref() != Some(&cat_str) {
            println!("  {} {}", "▸".blue(), cat_str.to_uppercase().blue().bold());
            current_category = Some(cat_str);
        }

        let status = if !info.enabled {
            "disabled".dimmed().to_string()
        } else if info.requires_key && !info.key_configured {
            "needs key".yellow().to_string()
        } else {
            "ready".green().to_string()
        };

        let key_icon = if info.requires_key { "🔑" } else { "  " };
        println!(
            "    {} {:<25} [{}] ({})",
            key_icon, info.name, status, info.rate_limit
        );
    }
}

fn cmd_config(action: ConfigAction, mut config: Config) {
    match action {
        ConfigAction::Set { source, key } => {
            config.set_api_key(source.clone(), key);
            if let Err(e) = config.save() {
                eprintln!("{}: {}", "Error saving config".red(), e);
                std::process::exit(1);
            }
            println!("{} API key set for '{}'", "✓".green(), source);
        }
        ConfigAction::Show => {
            let path = Config::config_path();
            println!("{}: {}", "Config path".bold(), path.display());
            println!();
            if path.exists() {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                println!("{content}");
            } else {
                println!(
                    "{}",
                    "No config file found. Run 'omnisearch config init' to create one.".yellow()
                );
            }
        }
        ConfigAction::Init => {
            let path = Config::config_path();
            if path.exists() {
                eprintln!(
                    "{}: Config file already exists at {}",
                    "Warning".yellow(),
                    path.display()
                );
                std::process::exit(1);
            }
            let dir = Config::config_dir();
            std::fs::create_dir_all(&dir).unwrap_or_else(|e| {
                eprintln!("{}: {}", "Error creating config dir".red(), e);
                std::process::exit(1);
            });
            std::fs::write(&path, Config::generate_template()).unwrap_or_else(|e| {
                eprintln!("{}: {}", "Error writing config".red(), e);
                std::process::exit(1);
            });
            println!("{} Config created at {}", "✓".green(), path.display());
        }
    }
}

async fn cmd_search(query: String, cli: &Cli, config: Config) {
    let timeout_secs = cli.timeout.unwrap_or(config.general.timeout_secs);
    let max_results = cli.max_results.unwrap_or(config.general.max_results);
    let format = &cli.format;

    let categories: Option<Vec<Category>> = cli.categories.as_ref().map(|cats| {
        cats.iter()
            .filter_map(|c| c.parse::<Category>().ok())
            .collect()
    });

    let client = build_client(timeout_secs);
    let sources = build_sources(&config);

    let mut rate_limiters = RateLimiterRegistry::new();
    for source in &sources {
        rate_limiters.register(source.name(), source.rate_limit_per_minute());
    }

    let engine = SearchEngine::new(client, sources, rate_limiters, timeout_secs);

    let search_query = SearchQuery {
        query: query.clone(),
        max_results,
        categories,
        sources: cli.sources.clone(),
    };

    let (results, errors, sources_queried) = engine.search(&search_query).await;
    let results = dedup_and_rank(results);
    let results: Vec<_> = results.into_iter().take(max_results).collect();

    let response = SearchResponse {
        query,
        total_results: results.len(),
        sources_queried,
        results,
        errors,
    };

    println!("{}", format_output(&response, format));
}
