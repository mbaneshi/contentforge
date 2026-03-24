# Installation

## System Requirements

- **Operating System:** macOS, Linux, or Windows
- **Rust:** 1.80+ (only needed for building from source)
- **Disk Space:** ~20 MB for the binary

No external services (databases, Redis, Docker) are required. ContentForge uses an embedded SQLite database.

## Install via Homebrew (macOS / Linux)

The recommended installation method for macOS and Linux:

```bash
brew install mbaneshi/tap/contentforge
```

Verify the installation:

```bash
contentforge --version
```

## Install via Cargo

If you have Rust installed:

```bash
cargo install contentforge
```

This builds the release binary and installs it to `~/.cargo/bin/`.

## Download Pre-built Binaries

Pre-built binaries are available for each release on the [GitHub Releases page](https://github.com/mbaneshi/contentforge/releases).

| Platform         | Binary                                    |
|------------------|-------------------------------------------|
| macOS (Apple Silicon) | `contentforge-aarch64-apple-darwin.tar.gz` |
| macOS (Intel)    | `contentforge-x86_64-apple-darwin.tar.gz`  |
| Linux (x86_64)   | `contentforge-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (ARM64)    | `contentforge-aarch64-unknown-linux-gnu.tar.gz` |
| Windows (x86_64) | `contentforge-x86_64-pc-windows-msvc.zip`  |

Download, extract, and place the binary in your PATH:

```bash
# Example for macOS Apple Silicon
curl -LO https://github.com/mbaneshi/contentforge/releases/latest/download/contentforge-aarch64-apple-darwin.tar.gz
tar xzf contentforge-aarch64-apple-darwin.tar.gz
mv contentforge /usr/local/bin/
```

## Build from Source

Clone the repository and build in release mode:

```bash
git clone https://github.com/mbaneshi/contentforge.git
cd contentforge
cargo build --release
```

The binary is at `target/release/contentforge`. Copy it to a directory in your PATH:

```bash
cp target/release/contentforge /usr/local/bin/
```

### Build with Frontend (Optional)

To include the SvelteKit Web UI in the binary, build the frontend first:

```bash
cd frontend
pnpm install
pnpm build
cd ..
cargo build --release
```

The frontend build output is embedded into the binary via `rust-embed`.

## Shell Completions

Generate shell completions for your shell:

```bash
# Bash
contentforge completions bash > ~/.local/share/bash-completion/completions/contentforge

# Zsh
contentforge completions zsh > ~/.zfunc/_contentforge

# Fish
contentforge completions fish > ~/.config/fish/completions/contentforge.fish
```

## Verify Installation

Run the doctor command to check that everything is working:

```bash
contentforge doctor
```

This verifies:

- Binary version and build info
- Database initialization
- Configuration file detection
- Platform credential status

## Next Steps

- [Quick Start tutorial](quickstart.md) -- publish your first content in 5 minutes
- [Configuration guide](configuration.md) -- set up platform credentials
