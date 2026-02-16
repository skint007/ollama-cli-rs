use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ollama-cli",
    about = "Manage remote Ollama instances",
    version
)]
pub struct Cli {
    /// Override the default Ollama URL
    #[arg(short = 'u', long = "url", global = true)]
    pub url: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all models on the remote instance
    List,

    /// Browse available models from ollama.com
    Library {
        /// Sort by "popular" or "newest"
        #[arg(short = 's', long, default_value = "popular")]
        sort: String,
        /// Search for models matching term
        #[arg(short = 'q', long)]
        search: Option<String>,
        /// Limit number of results
        #[arg(short = 'l', long, default_value_t = 20)]
        limit: usize,
        /// Show detailed information
        #[arg(short = 'v', long)]
        verbose: bool,
    },

    /// List currently running/loaded models
    Ps,

    /// Pull/download a model with progress
    Pull {
        /// Model name (e.g., gemma2:9b)
        model: String,
    },

    /// Remove a model from the remote instance
    Remove {
        /// Model name to remove
        model: String,
    },

    /// Unload a model from memory
    Unload {
        /// Model name to unload
        model: String,
    },

    /// Show detailed model information
    Show {
        /// Model name
        model: String,
        /// Include verbose information
        #[arg(short = 'v', long)]
        verbose: bool,
    },

    /// Copy a model to a new name
    Copy {
        /// Source model name
        source: String,
        /// Destination model name
        destination: String,
    },

    /// Create a new model
    Create {
        /// Name for the new model
        name: Option<String>,
        /// Base model to create from
        #[arg(long)]
        from: Option<String>,
        /// Path to Modelfile
        #[arg(short = 'f', long)]
        modelfile: Option<String>,
        /// Direct JSON payload file
        #[arg(short = 'j', long = "json")]
        json_file: Option<String>,
        /// Quantization type (q4_0, q4_1, q5_0, q5_1, q8_0, etc.)
        #[arg(short = 'q', long)]
        quantize: Option<String>,
        /// Disable streaming
        #[arg(long)]
        no_stream: bool,
    },

    /// Push a model to registry
    Push {
        /// Model name (registry reference)
        model: String,
        /// Allow insecure connections
        #[arg(long)]
        insecure: bool,
        /// Disable streaming
        #[arg(long)]
        no_stream: bool,
    },

    /// Generate text with a model
    Generate {
        /// Model name (positional)
        #[arg(value_name = "MODEL")]
        pos_model: Option<String>,
        /// Prompt text (positional)
        #[arg(value_name = "PROMPT")]
        pos_prompt: Option<String>,
        /// Model name (flag, overrides positional)
        #[arg(short = 'm', long = "model")]
        flag_model: Option<String>,
        /// Prompt text (flag, overrides positional)
        #[arg(short = 'p', long = "prompt")]
        flag_prompt: Option<String>,
        /// System message
        #[arg(short = 's', long)]
        system: Option<String>,
        /// Read prompt from file
        #[arg(short = 'f', long = "file")]
        file: Option<String>,
        /// Disable streaming
        #[arg(long)]
        no_stream: bool,
        /// Response format (e.g., json)
        #[arg(long)]
        format: Option<String>,
        /// Model options as JSON string
        #[arg(long)]
        options: Option<String>,
        /// Prompt template
        #[arg(long)]
        template: Option<String>,
        /// Context from previous response
        #[arg(long)]
        context: Option<String>,
        /// Use raw mode (bypass template)
        #[arg(long)]
        raw: bool,
    },

    /// Chat with a model (interactive/single message)
    Chat {
        /// Model name (positional)
        #[arg(value_name = "MODEL")]
        pos_model: Option<String>,
        /// Message (positional)
        #[arg(value_name = "MESSAGE")]
        pos_message: Option<String>,
        /// Model name (flag, overrides positional)
        #[arg(short = 'm', long = "model")]
        flag_model: Option<String>,
        /// Interactive chat mode
        #[arg(short = 'i', long)]
        interactive: bool,
        /// System message
        #[arg(short = 's', long)]
        system: Option<String>,
        /// Single message (flag)
        #[arg(long)]
        message: Option<String>,
        /// Read message from file
        #[arg(short = 'f', long = "file")]
        file: Option<String>,
        /// Disable streaming
        #[arg(long)]
        no_stream: bool,
        /// Response format
        #[arg(long)]
        format: Option<String>,
        /// Model options as JSON string
        #[arg(long)]
        options: Option<String>,
    },

    /// Generate embeddings from text
    Embed {
        /// Model name (positional)
        #[arg(value_name = "MODEL")]
        pos_model: Option<String>,
        /// Input text (positional)
        #[arg(value_name = "INPUT")]
        pos_input: Option<String>,
        /// Model name (flag, overrides positional)
        #[arg(short = 'm', long = "model")]
        flag_model: Option<String>,
        /// Input text (flag, overrides positional)
        #[arg(short = 'i', long = "input")]
        flag_input: Option<String>,
        /// Read input from file
        #[arg(short = 'f', long = "file")]
        file: Option<String>,
        /// Don't truncate long input
        #[arg(long)]
        no_truncate: bool,
        /// Model options as JSON string
        #[arg(long)]
        options: Option<String>,
    },

    /// Compare performance of multiple models
    Benchmark {
        /// Models to benchmark
        #[arg(required = true, num_args = 1..)]
        models: Vec<String>,
        /// Custom prompt for testing
        #[arg(short = 'p', long)]
        prompt: Option<String>,
        /// Number of rounds per model
        #[arg(short = 'r', long, default_value_t = 1)]
        rounds: u32,
        /// Save results to CSV file
        #[arg(long)]
        csv: bool,
        /// Don't unload models between tests
        #[arg(long)]
        no_unload: bool,
    },

    /// Show or set configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Launch interactive TUI
    Tui,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// List all configured URLs
    List,
    /// Add a named URL configuration
    Add {
        /// Name for this URL
        name: String,
        /// URL value
        url: String,
    },
    /// Switch to a named URL
    Use {
        /// Profile name to activate
        name: String,
    },
    /// Remove a named URL
    Remove {
        /// Profile name to remove
        name: String,
    },
    /// Set a configuration value (legacy)
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Reset to default configuration
    Reset,
}

/// Resolve model name from flag (takes precedence) or positional argument
pub fn resolve_model(flag: &Option<String>, positional: &Option<String>) -> Result<String> {
    flag.clone()
        .or_else(|| positional.clone())
        .ok_or_else(|| anyhow::anyhow!("Model name is required"))
}

/// Resolve text input from flag, positional, or file
pub fn resolve_text(
    flag: &Option<String>,
    positional: &Option<String>,
    file: &Option<String>,
) -> Result<Option<String>> {
    if let Some(text) = flag {
        return Ok(Some(text.clone()));
    }
    if let Some(text) = positional {
        return Ok(Some(text.clone()));
    }
    if let Some(path) = file {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path))?;
        return Ok(Some(content));
    }
    Ok(None)
}
