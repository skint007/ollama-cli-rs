use anyhow::Result;

use crate::api::types::CopyRequest;
use crate::client::OllamaClient;

pub async fn execute(client: &OllamaClient, source: &str, destination: &str) -> Result<()> {
    println!("Copying model: {} → {}", source, destination);
    println!("URL: {}", client.url());
    println!();

    let request = CopyRequest {
        source: source.to_string(),
        destination: destination.to_string(),
    };

    let response = client.post_raw("/api/copy", &request).await?;

    if response.status().is_success() {
        println!("✅ Model copied successfully: {} → {}", source, destination);
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to copy model (HTTP {}): {}", status, body);
    }

    Ok(())
}
