use anyhow::Result;
use comfy_table::{Cell, Table};

use crate::client::OllamaClient;
use crate::format::{bytes_to_human, format_datetime};

pub async fn execute(client: &OllamaClient) -> Result<()> {
    println!("Listing models from: {}", client.url());
    println!();

    let models = client.list_models().await?;

    if models.is_empty() {
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

    for model in &models {
        table.add_row(vec![
            Cell::new(&model.name),
            Cell::new(model.size.map(bytes_to_human).unwrap_or_else(|| "-".to_string())),
            Cell::new(
                model
                    .modified_at
                    .as_deref()
                    .map(format_datetime)
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ]);
    }

    println!("{table}");
    Ok(())
}
