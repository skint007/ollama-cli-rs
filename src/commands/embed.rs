use anyhow::Result;

use crate::api::types::{EmbedRequest, EmbedResponse};
use crate::cli::{resolve_model, resolve_text};
use crate::client::OllamaClient;

pub struct EmbedArgs {
    pub pos_model: Option<String>,
    pub pos_input: Option<String>,
    pub flag_model: Option<String>,
    pub flag_input: Option<String>,
    pub file: Option<String>,
    pub no_truncate: bool,
    pub options: Option<String>,
}

pub async fn execute(client: &OllamaClient, args: EmbedArgs) -> Result<()> {
    let model = resolve_model(&args.flag_model, &args.pos_model)?;
    let input = resolve_text(&args.flag_input, &args.pos_input, &args.file)?
        .ok_or_else(|| anyhow::anyhow!("Input text is required"))?;

    let options: Option<serde_json::Value> = args
        .options
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?;

    println!("Generating embeddings with model: {}", model);
    println!("URL: {}", client.url());
    println!("Input length: {} characters", input.len());
    println!();

    let request = EmbedRequest {
        model,
        input,
        truncate: !args.no_truncate,
        options,
    };

    let response: EmbedResponse = client.post("/api/embed", &request).await?;

    if response.embeddings.is_empty() {
        println!("No embeddings generated.");
        return Ok(());
    }

    println!("✅ Embeddings generated successfully");
    println!();

    let first = &response.embeddings[0];
    println!("Embedding dimensions: {}", first.len());
    println!("Number of embeddings: {}", response.embeddings.len());
    println!();

    // Preview first 10 values
    let preview_count = 10.min(first.len());
    let preview: Vec<String> = first[..preview_count]
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect();
    println!("Preview (first {} values):", preview_count);
    println!("{}", preview.join(", "));

    // Prompt for full output
    println!();
    let show_full = inquire::Confirm::new("Output full embeddings?")
        .with_default(false)
        .prompt()?;

    if show_full {
        println!();
        println!("{}", serde_json::to_string_pretty(&response.embeddings)?);
    }

    Ok(())
}
