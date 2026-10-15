use crate::types::SearchResponse;
use comfy_table::{Cell, Table};

pub fn format_output(response: &SearchResponse, format: &str) -> String {
    match format {
        "json" => format_json(response),
        "table" => format_table(response),
        "markdown" => format_markdown(response),
        _ => format_json(response),
    }
}

fn format_json(response: &SearchResponse) -> String {
    serde_json::to_string_pretty(response).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}

fn format_table(response: &SearchResponse) -> String {
    let mut table = Table::new();
    table.set_header(vec!["#", "Source", "Title", "URL"]);

    for (i, result) in response.results.iter().enumerate() {
        let title = truncate_chars(&result.title, 60);
        let url = truncate_chars(&result.url, 50);
        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&result.source),
            Cell::new(title),
            Cell::new(url),
        ]);
    }

    let mut output = format!(
        "Query: \"{}\" | {} results from {} sources\n",
        response.query,
        response.total_results,
        response.sources_queried.len()
    );
    output.push_str(&table.to_string());

    if !response.errors.is_empty() {
        output.push_str("\n\nErrors:\n");
        for err in &response.errors {
            output.push_str(&format!("  - {}: {}\n", err.source, err.error));
        }
    }

    output
}

/// Truncate on a char boundary (byte slicing panics on multi-byte titles).
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let head: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{head}...")
    } else {
        s.to_string()
    }
}

fn format_markdown(response: &SearchResponse) -> String {
    let mut md = format!(
        "# Search: \"{}\"\n\n**{} results** from {} sources\n\n",
        response.query,
        response.total_results,
        response.sources_queried.len()
    );

    for (i, result) in response.results.iter().enumerate() {
        md.push_str(&format!("## {}. {}\n", i + 1, result.title));
        md.push_str(&format!("**Source:** {} | **Category:** {}\n", result.source, result.category));
        if let Some(ref author) = result.author {
            md.push_str(&format!("**Author:** {}\n", author));
        }
        if let Some(ref published) = result.published {
            md.push_str(&format!("**Published:** {}\n", published.format("%Y-%m-%d")));
        }
        md.push_str(&format!("\n{}\n\n", result.snippet));
        md.push_str(&format!("[Link]({})\n\n---\n\n", result.url));
    }

    if !response.errors.is_empty() {
        md.push_str("## Errors\n\n");
        for err in &response.errors {
            md.push_str(&format!("- **{}**: {}\n", err.source, err.error));
        }
    }

    md
}
