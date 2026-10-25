use anyhow::Result;

use crate::api::types::GenerateRequest;
use crate::client::OllamaClient;

pub async fn execute(client: &OllamaClient, model: &str) -> Result<()> {
    println!("Unloading model: {}", model);
    println!("URL: {}", client.url());
    println!();

    let request = GenerateRequest {
        model: model.to_string(),
        prompt: String::new(),
        stream: false,
        keep_alive: Some(0),
        ..Default::default()
    };

    let response = client.post_raw("/api/generate", &request).await?;

    if response.status().is_success() {
        println!("✅ Model '{}' unloaded from memory", model);
        println!();
        println!("💡 The model files are still available and can be loaded again.");
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to unload model (HTTP {}): {}", status, body);
    }

    Ok(())
}
