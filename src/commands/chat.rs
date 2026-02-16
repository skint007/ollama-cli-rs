use anyhow::Result;
use std::io::Write;

use crate::api::types::{ChatMessage, ChatRequest, ChatResponse};
use crate::cli::{resolve_model, resolve_text};
use crate::client::OllamaClient;

pub struct ChatArgs {
    pub pos_model: Option<String>,
    pub pos_message: Option<String>,
    pub flag_model: Option<String>,
    pub interactive: bool,
    pub system: Option<String>,
    pub message: Option<String>,
    pub file: Option<String>,
    pub no_stream: bool,
    pub format: Option<String>,
    pub options: Option<String>,
}

pub async fn execute(client: &OllamaClient, args: ChatArgs) -> Result<()> {
    let model = resolve_model(&args.flag_model, &args.pos_model)?;

    let options: Option<serde_json::Value> = args
        .options
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?;

    let mut messages: Vec<ChatMessage> = Vec::new();

    if let Some(system) = &args.system {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system.clone(),
        });
    }

    if args.interactive {
        println!("💬 Starting chat with model: {}", model);
        println!("URL: {}", client.url());
        if args.system.is_some() {
            println!("System: {}", args.system.as_deref().unwrap());
        }
        println!();
        println!("Interactive chat mode (type 'exit', 'quit', or press Ctrl+C to quit)");
        println!("----------------------------------------");

        loop {
            print!("You: ");
            std::io::stdout().flush()?;

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() || input.is_empty() {
                break;
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            if matches!(input, "exit" | "quit" | "bye") {
                println!("👋 Goodbye!");
                break;
            }

            messages.push(ChatMessage {
                role: "user".to_string(),
                content: input.to_string(),
            });

            let request = ChatRequest {
                model: model.clone(),
                messages: messages.clone(),
                stream: !args.no_stream,
                format: args.format.clone(),
                options: options.clone(),
            };

            print!("Assistant: ");
            std::io::stdout().flush()?;

            let mut assistant_text = String::new();

            if !args.no_stream {
                let mut stream = client.post_stream("/api/chat", &request).await?;
                while let Some(chunk) = stream.next_json::<ChatResponse>().await? {
                    if let Some(err) = &chunk.error {
                        eprintln!("\nError: {}", err);
                        break;
                    }
                    if let Some(msg) = &chunk.message {
                        print!("{}", msg.content);
                        std::io::stdout().flush()?;
                        assistant_text.push_str(&msg.content);
                    }
                    if chunk.done == Some(true) {
                        println!();
                    }
                }
            } else {
                let response: ChatResponse = client.post("/api/chat", &request).await?;
                if let Some(msg) = &response.message {
                    println!("{}", msg.content);
                    assistant_text = msg.content.clone();
                }
            }

            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: assistant_text,
            });

            println!();
        }
    } else {
        // Single message mode
        let msg = resolve_text(&args.message, &args.pos_message, &args.file)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Message is required (use --interactive for chat mode, or provide a message)"
                )
            })?;

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: msg,
        });

        let request = ChatRequest {
            model: model.clone(),
            messages,
            stream: !args.no_stream,
            format: args.format,
            options,
        };

        if !args.no_stream {
            println!("🤖 Response:");
            println!("----------------------------------------");

            let mut stream = client.post_stream("/api/chat", &request).await?;
            while let Some(chunk) = stream.next_json::<ChatResponse>().await? {
                if let Some(err) = &chunk.error {
                    anyhow::bail!("Chat failed: {}", err);
                }
                if let Some(msg) = &chunk.message {
                    print!("{}", msg.content);
                    std::io::stdout().flush()?;
                }
                if chunk.done == Some(true) {
                    println!();
                    println!("----------------------------------------");
                }
            }
        } else {
            let response: ChatResponse = client.post("/api/chat", &request).await?;
            if let Some(err) = &response.error {
                anyhow::bail!("Chat failed: {}", err);
            }
            println!("🤖 Response:");
            println!("----------------------------------------");
            if let Some(msg) = &response.message {
                println!("{}", msg.content);
            }
            println!("----------------------------------------");
        }
    }

    Ok(())
}
