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
    let url = cli.url;

    match cli.command {
        Commands::List => {
            let client = connect(&cfg, url.as_deref()).await?;
            commands::list::execute(&client).await?;
        }
        Commands::Ps => {
            let client = connect(&cfg, url.as_deref()).await?;
            commands::ps::execute(&client).await?;
        }
        Commands::Pull { model } => {
            let client = connect(&cfg, url.as_deref()).await?;
            commands::pull::execute(&client, &model).await?;
        }
        Commands::Remove { model } => {
            let client = connect(&cfg, url.as_deref()).await?;
            commands::remove::execute(&client, &model).await?;
        }
        Commands::Unload { model } => {
            let client = connect(&cfg, url.as_deref()).await?;
            commands::unload::execute(&client, &model).await?;
        }
        Commands::Show { model, verbose } => {
            let client = connect(&cfg, url.as_deref()).await?;
            commands::show::execute(&client, &model, verbose).await?;
        }
        Commands::Copy {
            source,
            destination,
        } => {
            let client = connect(&cfg, url.as_deref()).await?;
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
            let client = connect(&cfg, url.as_deref()).await?;
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
            let client = connect(&cfg, url.as_deref()).await?;
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
            let client = connect(&cfg, url.as_deref()).await?;
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
            let client = connect(&cfg, url.as_deref()).await?;
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
            let client = connect(&cfg, url.as_deref()).await?;
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
            let client = connect(&cfg, url.as_deref()).await?;
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
            // A key problem shouldn't lock you out of the TUI's profile switcher:
            // start without the key and show the error instead
            let (client, startup_error) = match cfg
                .resolve_endpoint(url.as_deref())
                .and_then(|endpoint| OllamaClient::new(&endpoint))
            {
                Ok(client) => (client, None),
                Err(e) => (
                    OllamaClient::new(&cfg.resolve_endpoint_without_key(url.as_deref()))?,
                    Some(e.to_string()),
                ),
            };
            tui::run(client, cfg, startup_error).await?;
        }
    }

    Ok(())
}

/// Build a client for the effective endpoint and check it is reachable
async fn connect(cfg: &config::Config, url_override: Option<&str>) -> anyhow::Result<OllamaClient> {
    let client = OllamaClient::new(&cfg.resolve_endpoint(url_override)?)?;
    client.test_connection().await?;
    Ok(client)
}
