use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;

use crate::api::types::ProgressResponse;
use crate::client::OllamaClient;

pub async fn execute(client: &OllamaClient, model: &str) -> Result<()> {
    println!("Pulling model: {}", model);
    println!("URL: {}", client.url());
    println!();

    let body = serde_json::json!({
        "name": model,
        "stream": true
    });

    let mut stream = client.post_stream("/api/pull", &body).await?;
    let multi = MultiProgress::new();
    let mut bars: HashMap<String, ProgressBar> = HashMap::new();
    let mut spinner: Option<ProgressBar> = None;
    let mut pull_success = false;

    let style = ProgressStyle::with_template(
        "  📦 {prefix:.cyan} [{bar:40.green/dim}] {bytes}/{total_bytes} {percent}% {bytes_per_sec} ETA {eta}",
    )?
    .progress_chars("█░ ");

    let spinner_style = ProgressStyle::with_template("  {spinner:.cyan} {msg}")?.tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");

    while let Some(chunk) = stream.next_json::<ProgressResponse>().await? {
        if let Some(err) = &chunk.error {
            if let Some(sp) = spinner.take() {
                sp.finish_and_clear();
            }
            anyhow::bail!("❌ Pull failed: {}", err);
        }

        match (chunk.digest.as_deref(), chunk.total, chunk.completed) {
            (Some(digest), Some(total), Some(completed)) if total > 0 => {
                // Clear any active spinner when progress resumes
                if let Some(sp) = spinner.take() {
                    sp.finish_and_clear();
                }

                let bar = bars.entry(digest.to_string()).or_insert_with(|| {
                    let pb = multi.add(ProgressBar::new(total));
                    pb.set_style(style.clone());
                    let short_id = if digest.len() > 15 {
                        &digest[7..15]
                    } else {
                        digest
                    };
                    pb.set_prefix(format!("Layer {}", short_id));
                    pb
                });
                bar.set_position(completed);
                if completed >= total {
                    bar.finish();
                }
            }
            _ => {
                if let Some(status) = &chunk.status {
                    // Clear previous spinner before showing a new status
                    if let Some(sp) = spinner.take() {
                        sp.finish_and_clear();
                    }

                    match status.as_str() {
                        "pulling manifest" => {
                            let sp = multi.add(ProgressBar::new_spinner());
                            sp.set_style(spinner_style.clone());
                            sp.set_message("📋 Pulling manifest...");
                            sp.enable_steady_tick(std::time::Duration::from_millis(80));
                            spinner = Some(sp);
                        }
                        "verifying sha256 digest" => {
                            let sp = multi.add(ProgressBar::new_spinner());
                            sp.set_style(spinner_style.clone());
                            sp.set_message("🔍 Verifying integrity...");
                            sp.enable_steady_tick(std::time::Duration::from_millis(80));
                            spinner = Some(sp);
                        }
                        "writing manifest" => {
                            let sp = multi.add(ProgressBar::new_spinner());
                            sp.set_style(spinner_style.clone());
                            sp.set_message("📝 Writing manifest...");
                            sp.enable_steady_tick(std::time::Duration::from_millis(80));
                            spinner = Some(sp);
                        }
                        "removing any unused layers" => {
                            let sp = multi.add(ProgressBar::new_spinner());
                            sp.set_style(spinner_style.clone());
                            sp.set_message("🧹 Cleaning up unused layers...");
                            sp.enable_steady_tick(std::time::Duration::from_millis(80));
                            spinner = Some(sp);
                        }
                        "success" => {
                            multi.println("✅ Model pulled successfully!")?;
                            pull_success = true;
                        }
                        s if s.starts_with("pulling") => {}
                        s if !s.is_empty() => {
                            let sp = multi.add(ProgressBar::new_spinner());
                            sp.set_style(spinner_style.clone());
                            sp.set_message(format!("📌 {}", s));
                            sp.enable_steady_tick(std::time::Duration::from_millis(80));
                            spinner = Some(sp);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Clean up any remaining spinner
    if let Some(sp) = spinner.take() {
        sp.finish_and_clear();
    }

    println!();
    if pull_success {
        println!("✅ Model {} is ready to use!", model);
    } else {
        println!("ℹ️  Model {} is available (may have been already downloaded)", model);
    }

    Ok(())
}
