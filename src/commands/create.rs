use anyhow::{Context, Result};

use crate::api::types::{CreateRequest, CreateResponse};
use crate::client::OllamaClient;

pub struct CreateArgs {
    pub name: Option<String>,
    pub from: Option<String>,
    pub modelfile: Option<String>,
    pub json_file: Option<String>,
    pub quantize: Option<String>,
    pub no_stream: bool,
}

pub async fn execute(client: &OllamaClient, args: CreateArgs) -> Result<()> {
    // If JSON file provided, send it directly
    if let Some(json_path) = &args.json_file {
        let json_content = std::fs::read_to_string(json_path)
            .with_context(|| format!("Failed to read JSON file: {}", json_path))?;
        let payload: serde_json::Value = serde_json::from_str(&json_content)
            .with_context(|| format!("Failed to parse JSON file: {}", json_path))?;

        let name = args
            .name
            .or_else(|| payload.get("name").and_then(|v| v.as_str()).map(String::from))
            .ok_or_else(|| anyhow::anyhow!("Model name is required"))?;

        println!("Creating model: {}", name);
        println!("URL: {}", client.url());
        println!();

        let response = client.post_raw("/api/create", &payload).await?;
        if response.status().is_success() {
            println!("✅ Model created successfully: {}", name);
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Create failed (HTTP {}): {}", status, body);
        }
        return Ok(());
    }

    let name = args
        .name
        .ok_or_else(|| anyhow::anyhow!("Model name is required"))?;

    // Read modelfile content if provided
    let modelfile_content = if let Some(path) = &args.modelfile {
        Some(
            std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read Modelfile: {}", path))?,
        )
    } else {
        None
    };

    println!("Creating model: {}", name);
    println!("URL: {}", client.url());
    if let Some(from) = &args.from {
        println!("From: {}", from);
    }
    if let Some(q) = &args.quantize {
        println!("Quantize: {}", q);
    }
    println!();

    let request = CreateRequest {
        name: name.clone(),
        stream: !args.no_stream,
        from: args.from,
        modelfile: modelfile_content,
        quantize: args.quantize,
    };

    if !args.no_stream {
        let mut stream = client.post_stream("/api/create", &request).await?;

        while let Some(chunk) = stream.next_json::<CreateResponse>().await? {
            if let Some(err) = &chunk.error {
                anyhow::bail!("❌ Create failed: {}", err);
            }
            if let Some(status) = &chunk.status {
                println!("📌 {}", status);
            }
        }

        println!();
        println!("✅ Model created successfully: {}", name);
    } else {
        let response = client.post_raw("/api/create", &request).await?;
        if response.status().is_success() {
            println!("✅ Model created successfully: {}", name);
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Create failed (HTTP {}): {}", status, body);
        }
    }

    Ok(())
}
