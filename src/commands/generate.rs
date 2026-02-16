use anyhow::Result;
use std::io::Write;

use crate::api::types::{GenerateRequest, GenerateResponse};
use crate::cli::{resolve_model, resolve_text};
use crate::client::OllamaClient;
use crate::format::{nanos_to_secs, tokens_per_sec};

pub struct GenerateArgs {
    pub pos_model: Option<String>,
    pub pos_prompt: Option<String>,
    pub flag_model: Option<String>,
    pub flag_prompt: Option<String>,
    pub system: Option<String>,
    pub file: Option<String>,
    pub no_stream: bool,
    pub format: Option<String>,
    pub options: Option<String>,
    pub template: Option<String>,
    pub context: Option<String>,
    pub raw: bool,
}

pub async fn execute(client: &OllamaClient, args: GenerateArgs) -> Result<()> {
    let model = resolve_model(&args.flag_model, &args.pos_model)?;
    let prompt = resolve_text(&args.flag_prompt, &args.pos_prompt, &args.file)?
        .ok_or_else(|| anyhow::anyhow!("Prompt is required"))?;

    let options: Option<serde_json::Value> = args
        .options
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?;

    let context: Option<serde_json::Value> = args
        .context
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?;

    println!("Generating response with model: {}", model);
    println!("URL: {}", client.url());
    println!("Stream: {}", !args.no_stream);
    println!();

    let request = GenerateRequest {
        model,
        prompt,
        stream: !args.no_stream,
        system: args.system,
        format: args.format,
        options,
        template: args.template,
        context,
        raw: args.raw,
        keep_alive: None,
    };

    if request.stream {
        println!("🤖 Response:");
        println!("----------------------------------------");

        let mut stream = client.post_stream("/api/generate", &request).await?;

        while let Some(chunk) = stream.next_json::<GenerateResponse>().await? {
            if let Some(err) = &chunk.error {
                anyhow::bail!("Generation failed: {}", err);
            }
            if let Some(text) = &chunk.response {
                print!("{}", text);
                std::io::stdout().flush()?;
            }
            if chunk.done == Some(true) {
                println!();
                println!("----------------------------------------");
                if let (Some(count), Some(dur)) = (chunk.eval_count, chunk.eval_duration) {
                    println!("📊 Tokens: {}", count);
                    println!("⚡ Speed: {:.2} tokens/sec", tokens_per_sec(count, dur));
                }
                if let Some(total) = chunk.total_duration {
                    println!("⏱️  Total time: {:.2}s", nanos_to_secs(total));
                }
            }
        }
    } else {
        let response: GenerateResponse = client.post("/api/generate", &request).await?;

        if let Some(err) = &response.error {
            anyhow::bail!("Generation failed: {}", err);
        }

        println!("🤖 Response:");
        println!("----------------------------------------");
        if let Some(text) = &response.response {
            println!("{}", text);
        }
        println!("----------------------------------------");

        if let (Some(count), Some(dur)) = (response.eval_count, response.eval_duration) {
            println!("📊 Tokens: {}", count);
            println!("⚡ Speed: {:.2} tokens/sec", tokens_per_sec(count, dur));
        }
        if let Some(total) = response.total_duration {
            println!("⏱️  Total time: {:.2}s", nanos_to_secs(total));
        }
    }

    Ok(())
}
