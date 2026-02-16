use anyhow::Result;

use crate::api::types::{ShowRequest, ShowResponse};
use crate::client::OllamaClient;

pub async fn execute(client: &OllamaClient, model: &str, verbose: bool) -> Result<()> {
    println!("Model information for: {}", model);
    println!("URL: {}", client.url());
    println!();

    let request = ShowRequest {
        name: model.to_string(),
        verbose,
    };

    let response: ShowResponse = client.post("/api/show", &request).await?;

    if let Some(info) = &response.model_info {
        if let Some(name) = info.get("general.name").and_then(|v| v.as_str()) {
            println!("Model: {}", name);
        }
        if let Some(arch) = info.get("general.architecture").and_then(|v| v.as_str()) {
            println!("Architecture: {}", arch);
        }
        if let Some(params) = info.get("general.parameter_count").and_then(|v| v.as_u64()) {
            println!("Parameters: {}", format_param_count(params));
        }
        if let Some(quant) = info
            .get("general.quantization_version")
            .and_then(|v| v.as_u64())
        {
            println!("Quantization Version: {}", quant);
        }
        if let Some(ft) = info.get("general.file_type").and_then(|v| v.as_u64()) {
            println!("File Type: {}", ft);
        }
    }

    if let Some(template) = &response.template {
        if !template.is_empty() {
            println!();
            println!("Template:");
            println!("{}", template);
        }
    }

    if let Some(parameters) = &response.parameters {
        if !parameters.is_empty() {
            println!();
            println!("Parameters:");
            for line in parameters.lines() {
                println!("  {}", line);
            }
        }
    }

    if verbose {
        if let Some(modelfile) = &response.modelfile {
            if !modelfile.is_empty() {
                println!();
                println!("Modelfile:");
                println!("{}", modelfile);
            }
        }
    }

    Ok(())
}

fn format_param_count(count: u64) -> String {
    if count >= 1_000_000_000 {
        format!("{:.1}B", count as f64 / 1_000_000_000.0)
    } else if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}
