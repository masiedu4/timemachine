# TimeMachine

A powerful file versioning tool that helps track and manage file changes over time. TimeMachine creates snapshots of directories and allows you to track, restore, and manage changes efficiently.

## Features

- 📸 **Directory Snapshots**: Create point-in-time snapshots of your directories
- 🔍 **Change Tracking**: Track modifications, additions, and deletions
- ⏮️ **Time Travel**: Restore to any previous snapshot state
- 💾 **Space Efficient**: Deduplicates content for minimal storage usage
- 🔒 **Safe Operations**: Dry-run support and permission checks
- 🌐 **Cross-Platform**: Supports Windows, macOS, and Linux

## Installation

### macOS
```bash
# Coming soon to Homebrew
brew install timemachine
```

### Linux
```bash
# For x86_64 systems
curl -LO https://github.com/masiedu4/timemachine/releases/latest/download/timemachine-linux-amd64.tar.gz
tar xzf timemachine-linux-amd64.tar.gz
sudo mv timemachine /usr/local/bin/

# For ARM64 systems
curl -LO https://github.com/masiedu4/timemachine/releases/latest/download/timemachine-linux-arm64.tar.gz
tar xzf timemachine-linux-arm64.tar.gz
sudo mv timemachine /usr/local/bin/
```

### Windows
1. Download the latest release from [GitHub Releases](https://github.com/masiedu4/timemachine/releases)
2. Extract `timemachine-windows-amd64.exe`
3. Rename to `timemachine.exe`
4. Add to your PATH or move to a directory in your PATH

## Quick Start

1. Initialize a directory for tracking:
```bash
timemachine init ~/projects/my-app
```

2. Take a snapshot:
```bash
timemachine snapshot ~/projects/my-app
```

3. List snapshots:
```bash
timemachine list ~/projects/my-app
```

4. Check status:
```bash
timemachine status ~/projects/my-app
```

5. Restore to a previous snapshot:
```bash
# First, see what would change (dry-run)
timemachine restore ~/projects/my-app 1 --dry-run

# Then perform the actual restore
timemachine restore ~/projects/my-app 1
```

See [CLI Documentation](CLI.md) for detailed usage instructions.

## Building from Source

1. Install Rust (if you haven't already):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Clone and build:
```bash
git clone https://github.com/masiedu4/timemachine
cd timemachine
cargo build --release
```

The binary will be available at `target/release/timemachine`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the Apache-2.0 License - see the [LICENSE](LICENSE) file for details.
