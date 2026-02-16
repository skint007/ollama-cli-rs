use anyhow::Result;

use crate::cli::ConfigAction;
use crate::config::Config;

pub fn execute(action: Option<ConfigAction>, config: &mut Config) -> Result<()> {
    match action {
        None | Some(ConfigAction::Show) => show(config),
        Some(ConfigAction::List) => list(config),
        Some(ConfigAction::Add { name, url }) => add(config, &name, &url),
        Some(ConfigAction::Use { name }) => use_profile(config, &name),
        Some(ConfigAction::Remove { name }) => remove(config, &name),
        Some(ConfigAction::Set { key, value }) => set(config, &key, &value),
        Some(ConfigAction::Reset) => reset(),
    }
}

fn show(config: &Config) -> Result<()> {
    let config_path = Config::config_path()?;
    println!("Configuration file: {}", config_path.display());
    println!();
    println!("Current configuration:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let current = config
        .active_profile
        .as_deref()
        .unwrap_or("default");
    println!("Active profile: {}", current);
    println!("Active URL:     {}", config.resolve_url(None));
    println!();
    println!("Configured URLs:");

    if config.urls.is_empty() {
        println!("  (none)");
    } else {
        for (name, url) in &config.urls {
            if Some(name.as_str()) == config.active_profile.as_deref() {
                println!("  ➜ {}: {} (active)", name, url);
            } else {
                println!("    {}: {}", name, url);
            }
        }
    }

    Ok(())
}

fn list(config: &Config) -> Result<()> {
    if config.urls.is_empty() {
        println!("No URLs configured yet.");
        println!();
        println!("Add a URL with: ollama-cli config add <name> <url>");
        return Ok(());
    }

    println!("Configured URLs:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for (name, url) in &config.urls {
        if Some(name.as_str()) == config.active_profile.as_deref() {
            println!("➜ {}: {} (active)", name, url);
        } else {
            println!("  {}: {}", name, url);
        }
    }

    Ok(())
}

fn add(config: &mut Config, name: &str, url: &str) -> Result<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        anyhow::bail!("URL must start with http:// or https://");
    }

    if config.urls.contains_key(name) {
        println!("⚠️  URL '{}' already exists. Updating...", name);
    }

    config.urls.insert(name.to_string(), url.to_string());

    // If this is the first URL, make it active
    if config.active_profile.is_none() {
        config.active_profile = Some(name.to_string());
        config.default_url = url.to_string();
        config.save()?;
        println!("✅ Added URL '{}' and set as active", name);
    } else {
        config.save()?;
        println!("✅ Added URL '{}'", name);
        println!("   Use 'ollama-cli config use {}' to activate it", name);
    }

    Ok(())
}

fn use_profile(config: &mut Config, name: &str) -> Result<()> {
    if let Some(url) = config.urls.get(name) {
        let url = url.clone();
        config.active_profile = Some(name.to_string());
        config.default_url = url.clone();
        config.save()?;
        println!("✅ Switched to URL '{}'", name);
        println!("   URL: {}", url);
    } else {
        eprintln!("Error: URL '{}' not found", name);
        eprintln!();
        eprintln!("Available URLs:");
        for name in config.urls.keys() {
            eprintln!("  {}", name);
        }
        std::process::exit(1);
    }

    Ok(())
}

fn remove(config: &mut Config, name: &str) -> Result<()> {
    if !config.urls.contains_key(name) {
        anyhow::bail!("URL '{}' not found", name);
    }

    if config.active_profile.as_deref() == Some(name) {
        anyhow::bail!(
            "Cannot remove active URL '{}'\nSwitch to another URL first with: ollama-cli config use <name>",
            name
        );
    }

    config.urls.remove(name);
    config.save()?;
    println!("✅ Removed URL '{}'", name);

    Ok(())
}

fn set(config: &mut Config, key: &str, value: &str) -> Result<()> {
    match key {
        "OLLAMA_URL" | "url" => {
            config.default_url = value.to_string();
            config.save()?;
            println!("✅ Set OLLAMA_URL to: {}", value);
        }
        _ => {
            anyhow::bail!("Unknown configuration key '{}'\nAvailable keys: OLLAMA_URL, url", key);
        }
    }

    Ok(())
}

fn reset() -> Result<()> {
    let config_path = Config::config_path()?;
    if config_path.exists() {
        std::fs::remove_file(&config_path)?;
        println!("✅ Configuration reset to defaults");
    } else {
        println!("No configuration file to reset.");
    }
    Ok(())
}
