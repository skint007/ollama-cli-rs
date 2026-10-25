# ollama-cli

A CLI tool and interactive TUI for managing remote [Ollama](https://ollama.com) instances.

Built in Rust with [ratatui](https://github.com/ratatui/ratatui) for the terminal UI.

## Installation

### From GitHub Releases

Download the latest binary for your platform from the [Releases](https://github.com/skint007/ollama-cli-rs/releases) page.

| Platform | Asset |
|---|---|
| Linux (x86_64) | `ollama-cli-linux-amd64.tar.gz` |
| Linux (ARM64) | `ollama-cli-linux-arm64.tar.gz` |
| macOS (Intel) | `ollama-cli-macos-amd64.tar.gz` |
| macOS (Apple Silicon) | `ollama-cli-macos-arm64.tar.gz` |
| Windows (x86_64) | `ollama-cli-windows-amd64.zip` |

#### macOS Gatekeeper

macOS quarantines binaries downloaded from the internet. After extracting, remove the quarantine attribute before running:

```bash
xattr -d com.apple.quarantine ./ollama-cli
```

Alternatively, right-click the binary in Finder, select **Open**, and confirm the security prompt.

### From Source

```bash
git clone https://github.com/skint007/ollama-cli-rs.git
cd ollama-cli-rs
cargo build --release
# Binary is at target/release/ollama-cli
```

## Usage

```bash
ollama-cli <command> [options]
ollama-cli -u http://localhost:11434 <command>  # Override Ollama URL
```

### Interactive TUI

```bash
ollama-cli tui
```

Launch a full terminal interface with six sections:

| # | Section | Description |
|---|---|---|
| 1 | **Chat** | Chat with models, streaming responses with markdown rendering |
| 2 | **Models** | View, pull, copy, delete, and inspect installed models |
| 3 | **Running** | Monitor and unload currently loaded models |
| 4 | **Library** | Browse and pull models from ollama.com |
| 5 | **Benchmarks** | Compare model performance (tokens/sec, latency) |
| 6 | **Config** | Manage URL profiles for multiple Ollama instances |

Navigate sections with `Tab` / `1-6`. Press `?` for keybinding help in any section.

### CLI Commands

#### Model Management

```bash
ollama-cli list                          # List installed models
ollama-cli show <model> [-v]             # Show model details
ollama-cli pull <model>                  # Download a model
ollama-cli remove <model>                # Delete a model
ollama-cli copy <source> <dest>          # Copy a model
ollama-cli unload <model>                # Unload from memory
ollama-cli create --from <base> [-q q4_0] # Create/quantize a model
```

#### Chat & Generation

```bash
ollama-cli chat <model> "Hello"          # Single message
ollama-cli chat <model> -i               # Interactive chat session
ollama-cli generate <model> "prompt"     # Text generation
ollama-cli embed <model> "text"          # Generate embeddings
```

#### Discovery & Benchmarking

```bash
ollama-cli library                       # Browse ollama.com models
ollama-cli library -s newest -q "code"   # Search/sort library
ollama-cli benchmark model1 model2 -r 3  # Compare models (3 rounds)
ollama-cli benchmark model1 model2 --csv # Export results to CSV
```

#### Configuration

```bash
ollama-cli config show                   # Show current config
ollama-cli config add prod https://...   # Add a named profile
ollama-cli config add litellm http://host:4000 --kind openai --api-key-env LITELLM_API_KEY
ollama-cli config use prod               # Switch active profile
ollama-cli config list                   # List all profiles
ollama-cli config remove prod            # Remove a profile
ollama-cli config reset                  # Reset to defaults
```

## Configuration

Config is stored at `~/.config/ollama-cli/config.toml` (written with `0600` permissions):

```toml
default_url = "http://localhost:11434"
active_profile = "local"

[profiles.local]
url = "http://localhost:11434"
kind = "ollama"

[profiles.litellm]
url = "http://litellm.example.com:4000"   # with or without the /v1 suffix
kind = "openai"
api_key_env = "LITELLM_API_KEY"           # or: api_key = "sk-..."
```

**URL resolution order:** `--url` flag > active profile > default_url. A `--url` override is always treated as a plain Ollama URL with no API key.

Older configs with a `[urls]` table still load; those entries become `ollama` profiles and are rewritten as `[profiles.*]` the next time the config is saved.

### OpenAI-compatible backends (LiteLLM, etc.)

Profiles with `kind = "openai"` talk to `/v1/models`, `/v1/chat/completions` and `/v1/embeddings`, sending the API key as a `Bearer` token. `list`, `chat`, `generate`, `embed`, `benchmark` and the TUI chat/models views work as usual:

- `generate` runs through chat completions (`--system` becomes a system message); `--template`, `--context` and `--raw` are rejected.
- `--options` keys are sent as top-level params (`num_predict` is renamed to `max_tokens`); `--format json` maps to `response_format`.
- Tokens/sec is measured client-side. Streaming uses the backend's reported token count (`stream_options.include_usage`); non-streaming (including `benchmark`) divides by total request time, so it reads lower than Ollama's figure.

Ollama-only features (`pull`, `push`, `create`, `copy`, `rm`, `show`, `ps`, `unload`) report that they aren't supported. The TUI's "add profile" flow creates Ollama profiles; use `config add --kind openai` for OpenAI ones.

## License

MIT
