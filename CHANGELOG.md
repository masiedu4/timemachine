# Changelog

All notable changes to TimeMachine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.1.1] - 2025-01-02

### Added
- Added `--force` flag to restore command
  - Allows restoring snapshots even when there are uncommitted changes
  - Automatically creates a backup snapshot before forcing restore
  - Provides clear warning messages about the implications of force restore
- Automatic cleanup of orphaned content when it exceeds 100MB
- Smart cleanup system that preserves content shared between snapshots
- Size reporting for cleanup operations

### Changed
- Improved restore command error handling for uncommitted changes
- Enhanced restore output messages to be more informative
- Improved snapshot deletion to better handle shared content
- Enhanced space management with automatic orphaned content detection
- Added detailed feedback messages for cleanup operations

### Fixed
- Fixed cleanup logic that was incorrectly retaining content
- Ensured proper content cleanup when all snapshots are deleted

## [0.1.0] - 2024-12-31

### Added
- Initial release
- Basic snapshot functionality
- File change tracking
- Restore capabilities
- Space-efficient storage with deduplication
- Cross-platform support (Windows, macOS, Linux)
- Shell completion scripts
- Detailed CLI documentation
