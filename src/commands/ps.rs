use anyhow::Result;
use comfy_table::{Cell, Table};

use crate::api::types::PsResponse;
use crate::client::OllamaClient;
use crate::format::{bytes_to_human, format_datetime};

pub async fn execute(client: &OllamaClient) -> Result<()> {
    println!("Running models on: {}", client.url());
    println!();

    let response: PsResponse = client.get("/api/ps").await?;

    if response.models.is_empty() {
        println!("No models currently loaded.");
        println!();
        println!("💡 Use 'ollama-cli generate <model>' to load a model.");
        return Ok(());
    }

    println!("Currently loaded models:");
    println!();

    let mut table = Table::new();
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_header(vec![
        Cell::new("MODEL"),
        Cell::new("SIZE"),
        Cell::new("PROCESSOR"),
        Cell::new("EXPIRES"),
    ]);

    for model in &response.models {
        let family = model
            .details
            .as_ref()
            .and_then(|d| d.family.as_deref())
            .unwrap_or("-");
        let expires = model
            .expires_at
            .as_deref()
            .map(format_datetime)
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(&model.name),
            Cell::new(bytes_to_human(model.size)),
            Cell::new(family),
            Cell::new(expires),
        ]);
    }

    println!("{table}");
    println!();
    println!("📊 Total models loaded: {}", response.models.len());
    Ok(())
}
