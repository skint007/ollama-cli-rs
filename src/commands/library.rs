use anyhow::Result;
use scraper::{Html, Selector};

pub async fn execute(
    sort: &str,
    search: Option<&str>,
    limit: usize,
    verbose: bool,
) -> Result<()> {
    if sort != "popular" && sort != "newest" {
        anyhow::bail!("Sort must be 'popular' or 'newest'");
    }

    println!("📚 Fetching models from Ollama library...");
    println!("Sort: {}", sort);
    if let Some(q) = search {
        println!("Search: {}", q);
    }
    println!();

    let url = if sort == "newest" {
        "https://ollama.com/library?sort=newest".to_string()
    } else {
        "https://ollama.com/library".to_string()
    };

    let html_text = reqwest::get(&url).await?.text().await?;
    let document = Html::parse_document(&html_text);

    let link_selector =
        Selector::parse("a[href^='/library/']").expect("Invalid CSS selector");

    let mut displayed = 0;
    let mut seen = std::collections::HashSet::new();

    for element in document.select(&link_selector) {
        if displayed >= limit {
            break;
        }

        let href = element.value().attr("href").unwrap_or_default();
        let model_name = href.strip_prefix("/library/").unwrap_or(href);

        if model_name.is_empty() || model_name.contains('/') {
            continue;
        }

        if !seen.insert(model_name.to_string()) {
            continue;
        }

        // Apply search filter
        if let Some(q) = search {
            if !model_name.to_lowercase().contains(&q.to_lowercase()) {
                continue;
            }
        }

        displayed += 1;

        // Try to extract description from nested <p> elements
        let desc = element
            .select(&Selector::parse("p").unwrap())
            .next()
            .map(|p| p.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        // Extract size tags
        let tag_sel = Selector::parse("[x-test-size]").unwrap();
        let tags: Vec<String> = element
            .select(&tag_sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Extract capability tags
        let cap_sel = Selector::parse("[x-test-capability]").unwrap();
        let caps: Vec<String> = element
            .select(&cap_sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if verbose {
            println!("📦 \x1b[1m{}\x1b[0m", model_name);
            if !desc.is_empty() {
                println!("   Description: {}", desc);
            }
            if !tags.is_empty() {
                println!("   Tags:        {}", tags.join(" "));
            }
            if !caps.is_empty() {
                println!("   Features:    {}", caps.join(" "));
            }
            println!();
        } else {
            print!("📦 \x1b[1m{:<25}\x1b[0m", model_name);
            if !tags.is_empty() {
                print!(" [\x1b[36m{}\x1b[0m]", tags.join(" "));
            }
            if !desc.is_empty() {
                let max_len = 70;
                let truncated = if desc.len() > max_len {
                    format!("{}...", &desc[..max_len - 3])
                } else {
                    desc.clone()
                };
                print!(" {}", truncated);
            }
            println!();
        }
    }

    println!();
    if displayed == 0 {
        println!("No models found.");
        if search.is_some() {
            println!("Try a different search term or check https://ollama.com/library");
        }
    } else {
        println!("---");
        println!("Showing {} model(s)", displayed);
        println!();
        println!("💡 To pull a model: ollama-cli pull <model_name>");
    }

    Ok(())
}
