# TimeMachine

A powerful file versioning tool that allows you to track and manage changes in directories through snapshots. TimeMachine provides efficient file versioning with features like deduplication, change tracking, and point-in-time restoration.

## Features

- **Snapshot Management**: Create, list, and restore directory snapshots
- **Change Tracking**: Monitor file modifications, additions, and deletions
- **Efficient Storage**: Deduplication to minimize disk usage
- **Fast Restoration**: Quickly restore to any previous snapshot
- **Shell Completion**: Support for bash, zsh, fish, and PowerShell

## Installation

### Using Homebrew (Recommended for macOS)

```bash
brew tap masiedu4/timemachine
brew install timemachine
```

### Manual Installation

#### Prerequisites

- macOS or Linux: Make sure `/usr/local/bin` exists and is in your PATH
- Windows: Have a directory in your PATH ready for the executable

#### Step-by-Step Installation

1. **Download the correct binary for your system**:
   
   Visit the [releases page](https://github.com/masiedu4/timemachine/releases/latest) and download:

   - **macOS**:
     - Apple Silicon (M1/M2): `timemachine-macos-arm64.tar.gz`
     - Intel: `timemachine-macos-amd64.tar.gz`
   
   - **Linux**:
     - x86_64: `timemachine-linux-amd64.tar.gz`
     - ARM64: `timemachine-linux-arm64.tar.gz`
   
   - **Windows**:
     - x86_64: `timemachine-windows-amd64.zip`

2. **Install the binary**:

   **macOS/Linux**:
   ```bash
   # Navigate to downloads
   cd ~/Downloads

   # Extract the archive (replace * with your platform)
   tar -xzf timemachine-*.tar.gz

   # Verify binary exists
   ls -l timemachine

   # Move to PATH
   sudo mkdir -p /usr/local/bin
   sudo mv timemachine /usr/local/bin/
   ```

   **Windows**:
   ```powershell
   # Extract the ZIP file
   Expand-Archive timemachine-windows-amd64.zip -DestinationPath C:\Windows\System32
   ```

3. **Verify Installation**:
   ```bash
   # Should show the binary location
   which timemachine

   # Should show version 0.1.1
   timemachine --version
   ```

   If you see "command not found", try:
   1. Opening a new terminal window
   2. Verifying the binary is in your PATH
   3. Checking you downloaded the correct version for your system

### Shell Completions

TimeMachine supports shell completions for a better command-line experience:

```bash
# Generate completion scripts
timemachine completions

# Follow the printed instructions to install for your shell
```

## Quick Start

1. **Initialize a Directory**:
   ```bash
   timemachine init ~/Documents/my-project
   ```

2. **Create a Snapshot**:
   ```bash
   timemachine snapshot ~/Documents/my-project
   ```

3. **List Snapshots**:
   ```bash
   timemachine list ~/Documents/my-project
   ```

4. **Restore a Previous Version**:
   ```bash
   # List snapshots to get the ID
   timemachine list ~/Documents/my-project
   
   # Restore to a specific snapshot
   timemachine restore ~/Documents/my-project --id <snapshot-id>
   
   # Force restore (creates backup of current state)
   timemachine restore ~/Documents/my-project --id <snapshot-id> --force
   ```

## Documentation

For detailed usage instructions and examples, see:
- [CLI Documentation](CLI.md)
- [Changelog](CHANGELOG.md)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
