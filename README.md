# Tacoshell 🌮

[![CI](https://github.com/tacoshell/tacoshell/actions/workflows/ci.yml/badge.svg)](https://github.com/tacoshell/tacoshell/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Tacoshell is an open-source SSH Client built with Rust, Tauri, and React.

## About

Tacoshell is designed to provide a seamless and efficient SSH experience for users. It offers a user-friendly interface,
robust security features, and support for various platforms. Whether you're a developer, system administrator, or anyone
who needs to manage remote servers, Tacoshell aims to be your go-to SSH client.

## Features

- 🔐 **Secure Credential Storage** - Secrets encrypted with age, master key stored in OS keyring
- 🖥️ **Modern Terminal** - xterm.js-based terminal with full PTY support
- 📁 **SFTP Support** - Built-in file transfer capabilities (Phase 3)
- ☸️ **Kubernetes Integration** - Manage K8s clusters from the same interface (Phase 4)
- 🎨 **Beautiful UI** - Modern React-based interface with split-pane support
- 🚀 **Cross-Platform** - Works on Windows, macOS, and Linux

## Installation

### Pre-built Binaries

Download the latest release from the [Releases](https://github.com/tacoshell/tacoshell/releases) page.

### Build from Source

#### Prerequisites

- [Rust](https://rustup.rs/) (1.70 or later)
- [Node.js](https://nodejs.org/) (18 or later)
- Platform-specific dependencies:

**Windows:**
```powershell
# No additional dependencies needed
```

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install -y libssl-dev libsqlite3-dev libsecret-1-dev \
  libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
```

#### Building

```bash
# Clone the repository
git clone https://github.com/tacoshell/tacoshell.git
cd tacoshell

# Install frontend dependencies
cd ui && npm install && cd ..

# Build the application
make build

# Or run in development mode
make tauri-dev
```

## Development

### Project Structure

```
tacoshell/
├── crates/                    # Rust workspace crates
│   ├── tacoshell-core/        # Core types, traits, and error handling
│   ├── tacoshell-ssh/         # SSH client implementation
│   ├── tacoshell-storage/     # JSON storage layer
│   ├── tacoshell-secrets/     # Secret encryption/decryption
│   ├── tacoshell-transfer/    # SFTP/FTP file transfer
│   └── tacoshell-k8s/         # Kubernetes integration
├── ui/                        # Frontend application
│   ├── src/                   # React components and hooks
│   └── src-tauri/             # Tauri application backend
├── configs/                   # Configuration templates
├── docs/                      # Documentation and ADRs
└── scripts/                   # Build and utility scripts
```

### Available Commands

```bash
make build       # Build release binary
make dev         # Build debug binary
make test        # Run all tests
make lint        # Run clippy lints
make fmt         # Check formatting
make fmt-fix     # Auto-fix formatting
make check       # Run all checks (fmt + lint + test)
make doc         # Generate and open documentation
make tauri-dev   # Run Tauri development server
make tauri-build # Build Tauri application
```

### Running Tests

```bash
cargo test --all
```

## Roadmap

See [ROADMAP.md](ROADMAP.md) for detailed phases and future plans.

## Documentation

- [Architecture Decision Records (ADR)](docs/adr/0001-use-rust.md)
- [Design Documents](docs/README.md)

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting a PR.

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/amazing-feature`)
3. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/)
4. Push to the branch (`git push origin feat/amazing-feature`)
5. Open a Pull Request

### Commit Message Format

```
<type>(<scope>): <description>

Types: feat, fix, docs, style, refactor, test, chore
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Desktop app framework
- [xterm.js](https://xtermjs.org/) - Terminal emulator
- [age](https://age-encryption.org/) - Modern encryption
- [ssh2-rs](https://github.com/alexcrichton/ssh2-rs) - SSH bindings for Rust
