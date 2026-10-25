use anyhow::Result;

use crate::api::types::{error_message, DeleteRequest};
use crate::client::OllamaClient;

pub async fn execute(client: &OllamaClient, model: &str) -> Result<()> {
    client.require_ollama("Removing models")?;

    let confirm = inquire::Confirm::new(&format!("Are you sure you want to remove model '{}'?", model))
        .with_default(false)
        .prompt()?;

    if !confirm {
        println!("Operation cancelled.");
        return Ok(());
    }

    let request = DeleteRequest {
        name: model.to_string(),
    };

    let response = client.delete("/api/delete", &request).await?;

    if response.status().is_success() {
        println!("✅ Model '{}' removed successfully", model);
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to remove model (HTTP {}): {}", status, error_message(&body));
    }

    Ok(())
}
