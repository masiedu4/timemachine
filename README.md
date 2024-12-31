# TimeMachine

A powerful file versioning tool that allows you to track and manage changes in directories through snapshots. TimeMachine provides efficient file versioning with features like deduplication, change tracking, and point-in-time restoration.

## Features

- 📸 **Snapshot Management**: Create, list, and restore directory snapshots
- 🔄 **Change Tracking**: Monitor file modifications, additions, and deletions
- 💨 **Efficient Storage**: Deduplication to minimize disk usage
- ⚡️ **Fast Restoration**: Quickly restore to any previous snapshot
- 🛠 **Shell Completion**: Support for bash, zsh, fish, and PowerShell

## Installation

### Using Homebrew (macOS)

```bash
brew tap masiedu4/timemachine
brew install masiedu4/timemachine/timemachine
```

### Manual Installation

#### macOS
```bash
# Intel Mac
curl -LO https://github.com/masiedu4/timemachine/releases/download/v0.1.0/timemachine-macos-amd64.tar.gz
tar xf timemachine-macos-amd64.tar.gz
sudo mv timemachine /usr/local/bin/

# Apple Silicon Mac
curl -LO https://github.com/masiedu4/timemachine/releases/download/v0.1.0/timemachine-macos-arm64.tar.gz
tar xf timemachine-macos-arm64.tar.gz
sudo mv timemachine /usr/local/bin/
```

#### Linux
```bash
# x86_64
curl -LO https://github.com/masiedu4/timemachine/releases/download/v0.1.0/timemachine-linux-amd64.tar.gz
tar xf timemachine-linux-amd64.tar.gz
sudo mv timemachine /usr/local/bin/

# ARM64
curl -LO https://github.com/masiedu4/timemachine/releases/download/v0.1.0/timemachine-linux-arm64.tar.gz
tar xf timemachine-linux-arm64.tar.gz
sudo mv timemachine /usr/local/bin/
```

#### Windows
Download the appropriate zip file from the [releases page](https://github.com/masiedu4/timemachine/releases) and add it to your PATH.

## Quick Start

1. Initialize a directory for version tracking:
```bash
timemachine init ~/projects/my-app
```

2. Take a snapshot:
```bash
timemachine snapshot ~/projects/my-app
```

3. Check status:
```bash
timemachine status ~/projects/my-app
```

4. List snapshots:
```bash
timemachine list ~/projects/my-app
```

5. Restore to a previous snapshot:
```bash
timemachine restore ~/projects/my-app <snapshot-id>
```

For detailed usage instructions, see the [CLI Documentation](CLI.md).

## Building from Source

Requirements:
- Rust 1.70 or later
- Cargo

```bash
# Clone the repository
git clone https://github.com/masiedu4/timemachine
cd timemachine

# Build
cargo build --release

# Install
cargo install --path .
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
