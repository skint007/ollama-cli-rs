use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;

use crate::api::types::{ProgressResponse, PushRequest};
use crate::client::OllamaClient;

pub async fn execute(
    client: &OllamaClient,
    model: &str,
    insecure: bool,
    no_stream: bool,
) -> Result<()> {
    println!("Pushing model: {}", model);
    println!("URL: {}", client.url());
    println!();

    if no_stream {
        let request = PushRequest {
            name: model.to_string(),
            stream: false,
            insecure,
        };
        let response = client.post_raw("/api/push", &request).await?;
        if response.status().is_success() {
            println!("✅ Model pushed successfully!");
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Push failed (HTTP {}): {}", status, body);
        }
        return Ok(());
    }

    let request = PushRequest {
        name: model.to_string(),
        stream: true,
        insecure,
    };

    let mut stream = client.post_stream("/api/push", &request).await?;
    let multi = MultiProgress::new();
    let mut bars: HashMap<String, ProgressBar> = HashMap::new();

    let style = ProgressStyle::with_template(
        "  📤 {prefix:.cyan} [{bar:40.green/dim}] {bytes}/{total_bytes} {percent}% {bytes_per_sec} ETA {eta}",
    )?
    .progress_chars("█░ ");

    while let Some(chunk) = stream.next_json::<ProgressResponse>().await? {
        if let Some(err) = &chunk.error {
            anyhow::bail!("❌ Push failed: {}", err);
        }

        match (chunk.digest.as_deref(), chunk.total, chunk.completed) {
            (Some(digest), Some(total), Some(completed)) if total > 0 => {
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
                    match status.as_str() {
                        "pushing manifest" => {
                            multi.println("📋 Pushing manifest...")?;
                        }
                        s if !s.is_empty() => {
                            multi.println(format!("📌 {}", s))?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    println!();
    println!("✅ Model {} pushed successfully!", model);

    Ok(())
}
