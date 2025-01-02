# TimeMachine CLI Documentation

## Overview

TimeMachine is a command-line tool for creating and managing directory snapshots. It provides efficient file versioning with features like deduplication, change tracking, and point-in-time restoration.

## Installation

### Using Homebrew (recommended for macOS)
```bash
# Install
brew tap masiedu4/timemachine
brew install masiedu4/timemachine/timemachine

# Update
brew update
brew upgrade timemachine
```

For other installation methods, see the [README](README.md).

## Global Usage

```bash
timemachine [COMMAND] [OPTIONS] [ARGUMENTS]
```

## Commands

### init
Initialize a directory for version tracking.

```bash
timemachine init <DIRECTORY>
```

**Arguments:**
- `DIRECTORY`: Path to the directory to initialize (required)

**Example:**
```bash
# Initialize a project directory
timemachine init ~/projects/my-app

# Initialize current directory
timemachine init .
```

### snapshot
Create a new snapshot of the current directory state.

```bash
timemachine snapshot <DIRECTORY>
```

**Arguments:**
- `DIRECTORY`: Path to the directory to snapshot (required)

**Example:**
```bash
# Take a snapshot of a project
timemachine snapshot ~/projects/my-app

# Take a snapshot of current directory
timemachine snapshot .
```

### list
List all snapshots for a directory.

```bash
timemachine list <DIRECTORY> [--detailed]
```

**Arguments:**
- `DIRECTORY`: Path to the directory (required)
- `--detailed`: Show additional information including space usage

**Examples:**
```bash
# Basic listing
timemachine list ~/projects/my-app

# Detailed listing with space usage
timemachine list ~/projects/my-app --detailed
```

### status
Show the current status of a directory.

```bash
timemachine status <DIRECTORY>
```

**Arguments:**
- `DIRECTORY`: Path to the directory to check status (required)

**Example:**
```bash
# Check status of a project
timemachine status ~/projects/my-app

# Check status of current directory
timemachine status .
```

### diff
Compare two snapshots to see what has changed.

```bash
timemachine diff <DIRECTORY> <SNAPSHOT_ID_1> <SNAPSHOT_ID_2>
```

**Arguments:**
- `DIRECTORY`: Path to the directory (required)
- `SNAPSHOT_ID_1`: ID of the first snapshot to compare (required)
- `SNAPSHOT_ID_2`: ID of the second snapshot to compare (required)

**Example:**
```bash
# Compare snapshots 1 and 2
timemachine diff ~/projects/my-app 1 2

# Compare current directory snapshots
timemachine diff . 1 2
```

### restore
Restore a directory to a specific snapshot state.

```bash
timemachine restore <DIRECTORY> <SNAPSHOT_ID> [--dry-run] [--force]
```

**Arguments:**
- `DIRECTORY`: Path to the directory to restore (required)
- `SNAPSHOT_ID`: ID of the snapshot to restore to (required)

**Options:**
- `--dry-run`: Show what would be changed without making actual changes
- `--force`: Force restore even if there are uncommitted changes. This will:
  1. Create a backup snapshot of the current state
  2. Override any uncommitted changes
  3. Restore to the specified snapshot

**Examples:**
```bash
# Restore directory to snapshot #5
timemachine restore /path/to/dir 5

# Preview changes without applying them
timemachine restore /path/to/dir 5 --dry-run

# Force restore even with uncommitted changes
timemachine restore /path/to/dir 5 --force
```

### delete
Delete a specific snapshot.

```bash
timemachine delete <DIRECTORY> <SNAPSHOT_ID> [--cleanup]
```

**Arguments:**
- `DIRECTORY`: Path to the directory (required)
- `SNAPSHOT_ID`: ID of the snapshot to delete (required)
- `--cleanup`: Immediately remove content unique to this snapshot

**Cleanup Behavior:**
- Without `--cleanup`: Content is automatically cleaned up when:
  1. All snapshots are deleted
  2. Orphaned content exceeds 100MB
- With `--cleanup`: Immediately removes content unique to the deleted snapshot
- Space savings are reported after each cleanup operation

**Examples:**
```bash
# Delete a snapshot (automatic cleanup if orphaned content exceeds threshold)
timemachine delete ~/projects/my-app 2

# Delete and immediately clean up content unique to this snapshot
timemachine delete ~/projects/my-app 2 --cleanup
```

## Shell Completion

TimeMachine provides shell completion support for:
- Bash
- Zsh
- Fish
- PowerShell

To generate completion scripts:
```bash
timemachine completions [SHELL]
```

**Arguments:**
- `SHELL`: Optional shell name (bash, zsh, fish, powershell)

If no shell is specified, generates completions for all supported shells.

## Error Handling

Common error scenarios:

1. Directory Access:
   - Directory not found or inaccessible
   - Insufficient permissions

2. Snapshot Operations:
   - Snapshot not found
   - Invalid snapshot ID
   - Insufficient space

3. Restore Operations:
   - Uncommitted changes present
   - Insufficient space
   - Invalid restore point

## Best Practices

1. **Regular Snapshots**
   - Take snapshots at meaningful points (after major changes)
   - Use descriptive commit messages
   - Regular snapshots make restoration easier

2. **Space Management**
   - Regularly clean up old snapshots
   - Use `--cleanup` when deleting snapshots
   - Monitor available space with `status` command

3. **Safe Restoration**
   - Always use `--dry-run` first
   - Ensure sufficient space before restoration
   - Back up important files before large restores

4. **Performance**
   - Avoid tracking large binary files
   - Exclude temporary and build files
