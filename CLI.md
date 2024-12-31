# TimeMachine CLI Documentation

## Overview

TimeMachine is a command-line tool for creating and managing directory snapshots. It provides efficient file versioning with features like deduplication, change tracking, and point-in-time restoration.

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
timemachine init ~/projects/my-app
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
timemachine snapshot ~/projects/my-app
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
timemachine status ~/projects/my-app
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
timemachine diff ~/projects/my-app 1 2
```

### restore
Restore a directory to a specific snapshot state.

```bash
timemachine restore <DIRECTORY> <SNAPSHOT_ID> [--dry-run]
```

**Arguments:**
- `DIRECTORY`: Path to the directory to restore (required)
- `SNAPSHOT_ID`: ID of the snapshot to restore to (required)
- `--dry-run`: Show what would be changed without making actual changes

**Examples:**
```bash
# Preview changes
timemachine restore ~/projects/my-app 2 --dry-run

# Perform actual restore
timemachine restore ~/projects/my-app 2
```

### delete
Delete a specific snapshot.

```bash
timemachine delete <DIRECTORY> <SNAPSHOT_ID> [--cleanup]
```

**Arguments:**
- `DIRECTORY`: Path to the directory (required)
- `SNAPSHOT_ID`: ID of the snapshot to delete (required)
- `--cleanup`: Remove unused content after deletion

**Examples:**
```bash
# Delete snapshot
timemachine delete ~/projects/my-app 2

# Delete snapshot and clean up unused content
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
