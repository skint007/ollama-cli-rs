use anyhow::Result;

use crate::api::types::{DeleteRequest, ErrorResponse};
use crate::client::OllamaClient;

pub async fn execute(client: &OllamaClient, model: &str) -> Result<()> {
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
        let msg = serde_json::from_str::<ErrorResponse>(&body)
            .map(|e| e.error)
            .unwrap_or(body);
        anyhow::bail!("Failed to remove model (HTTP {}): {}", status, msg);
    }

    Ok(())
}
