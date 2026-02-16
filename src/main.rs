mod api;
mod cli;
mod client;
mod commands;
mod config;
mod format;
mod stream;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};
use client::OllamaClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut cfg = config::Config::load()?;
    let url = cfg.resolve_url(cli.url.as_deref());

    match cli.command {
        Commands::List => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::list::execute(&client).await?;
        }
        Commands::Ps => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::ps::execute(&client).await?;
        }
        Commands::Pull { model } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::pull::execute(&client, &model).await?;
        }
        Commands::Remove { model } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::remove::execute(&client, &model).await?;
        }
        Commands::Unload { model } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::unload::execute(&client, &model).await?;
        }
        Commands::Show { model, verbose } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::show::execute(&client, &model, verbose).await?;
        }
        Commands::Copy {
            source,
            destination,
        } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::copy::execute(&client, &source, &destination).await?;
        }
        Commands::Create {
            name,
            from,
            modelfile,
            json_file,
            quantize,
            no_stream,
        } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::create::execute(
                &client,
                commands::create::CreateArgs {
                    name,
                    from,
                    modelfile,
                    json_file,
                    quantize,
                    no_stream,
                },
            )
            .await?;
        }
        Commands::Push {
            model,
            insecure,
            no_stream,
        } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::push::execute(&client, &model, insecure, no_stream).await?;
        }
        Commands::Generate {
            pos_model,
            pos_prompt,
            flag_model,
            flag_prompt,
            system,
            file,
            no_stream,
            format,
            options,
            template,
            context,
            raw,
        } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::generate::execute(
                &client,
                commands::generate::GenerateArgs {
                    pos_model,
                    pos_prompt,
                    flag_model,
                    flag_prompt,
                    system,
                    file,
                    no_stream,
                    format,
                    options,
                    template,
                    context,
                    raw,
                },
            )
            .await?;
        }
        Commands::Chat {
            pos_model,
            pos_message,
            flag_model,
            interactive,
            system,
            message,
            file,
            no_stream,
            format,
            options,
        } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::chat::execute(
                &client,
                commands::chat::ChatArgs {
                    pos_model,
                    pos_message,
                    flag_model,
                    interactive,
                    system,
                    message,
                    file,
                    no_stream,
                    format,
                    options,
                },
            )
            .await?;
        }
        Commands::Embed {
            pos_model,
            pos_input,
            flag_model,
            flag_input,
            file,
            no_truncate,
            options,
        } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::embed::execute(
                &client,
                commands::embed::EmbedArgs {
                    pos_model,
                    pos_input,
                    flag_model,
                    flag_input,
                    file,
                    no_truncate,
                    options,
                },
            )
            .await?;
        }
        Commands::Benchmark {
            models,
            prompt,
            rounds,
            csv,
            no_unload,
        } => {
            let client = OllamaClient::new(&url);
            client.test_connection().await?;
            commands::benchmark::execute(
                &client,
                &models,
                prompt.as_deref(),
                rounds,
                csv,
                no_unload,
            )
            .await?;
        }
        Commands::Library {
            sort,
            search,
            limit,
            verbose,
        } => {
            commands::library::execute(&sort, search.as_deref(), limit, verbose).await?;
        }
        Commands::Config { action } => {
            commands::config_cmd::execute(action, &mut cfg)?;
        }
        Commands::Tui => {
            let client = OllamaClient::new(&url);
            tui::run(client, cfg).await?;
        }
    }

    Ok(())
}
