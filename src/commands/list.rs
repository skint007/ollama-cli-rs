use anyhow::Result;
use comfy_table::{Cell, Table};

use crate::api::types::TagsResponse;
use crate::client::OllamaClient;
use crate::format::{bytes_to_human, format_datetime};

pub async fn execute(client: &OllamaClient) -> Result<()> {
    println!("Listing models from: {}", client.url());
    println!();

    let response: TagsResponse = client.get("/api/tags").await?;

    if response.models.is_empty() {
        println!("No models found.");
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_header(vec![
        Cell::new("NAME"),
        Cell::new("SIZE"),
        Cell::new("MODIFIED"),
    ]);

    for model in &response.models {
        table.add_row(vec![
            Cell::new(&model.name),
            Cell::new(bytes_to_human(model.size)),
            Cell::new(format_datetime(&model.modified_at)),
        ]);
    }

    println!("{table}");
    Ok(())
}
