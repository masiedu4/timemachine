mod core;

use chrono::prelude::*;

use std::io::{ErrorKind};
use std::path::Path;
use std::{fs, io};

use core::models::{Snapshot, SnapshotComparison, SnapshotMetadata,RestoreReport};
use core::snapshot::{collect_file_states, create_file_map, find_deleted_files, find_modified_files, find_new_files, find_snapshot, load_all_snapshots};
use core::restore::{validate_permissions,generate_restore_report, has_available_space, has_uncommitted_changes, perform_restore};

pub fn initialize_timemachine(base_dir: &str) -> Result<(), io::Error> {
    let root_path = Path::new(base_dir);

    let timemachine = root_path.join(".timemachine");

    // create a new .timemachine folder if it does not exist
    if !timemachine.exists() {
        fs::create_dir_all(&timemachine)?;
    }

    let metadata_file = timemachine.join("metadata.json");

    // create a new metadata, a metadata.json file if it does not exist, and write to it
    if !metadata_file.exists() {
        let new_metadata = SnapshotMetadata {
            snapshots: Vec::new(),
        };

        let content = serde_json::to_string(&new_metadata)?;

        fs::write(&metadata_file, content.as_bytes())?;
    }

    Ok(())
}

pub fn take_snapshot(dir: &str) -> io::Result<()> {
    let base_path = Path::new(dir);
    let metadata_folder = base_path.join(".timemachine");
    let metadata_file = metadata_folder.join("metadata.json");

    // ensure that timestamp directory exists
    if !metadata_folder.exists() {
        eprintln!(
            "The directory '{}' is not initialized for snapshots. Initializing it now.",
            dir
        );

        initialize_timemachine(&dir)?;
    }

    // Load snapshots from metadata.json
    let mut metadata: SnapshotMetadata = {
        let content = fs::read_to_string(&metadata_file)?;
        serde_json::from_str(&content)?
    };

    let file_states = collect_file_states(&dir)?;


    let snapshot = Snapshot {
        id: metadata.snapshots.len() + 1,
        timestamp: Local::now().to_rfc3339(),
        changes: file_states.len(),
        file_states,
    };

    // update metadata
    metadata.snapshots.push(snapshot);
    let updated_metadata = serde_json::to_string_pretty(&metadata)?;
    fs::write(metadata_file, updated_metadata)?;

    Ok(())
}

pub fn differentiate_snapshots(
    path: &str,
    snapshot_id1: usize,
    snapshot_id2: usize,
) -> io::Result<SnapshotComparison> {
    let metadata = load_all_snapshots(path)?;

    let snapshot1 = find_snapshot(&metadata, snapshot_id1).ok_or_else(|| {
        io::Error::new(
            ErrorKind::NotFound,
            format!("Snapshot ID not found: {}", snapshot_id1),
        )
    })?;

    let snapshot2 = find_snapshot(&metadata, snapshot_id2).ok_or_else(|| {
        io::Error::new(
            ErrorKind::NotFound,
            format!("Snapshot ID not found: {}", snapshot_id2),
        )
    })?;

    let snapshot1_map = create_file_map(&snapshot1.file_states);
    let snapshot2_map = create_file_map(&snapshot2.file_states);

    let new_files = find_new_files(&snapshot2_map, &snapshot1_map);
    let modified_files = find_modified_files(&snapshot1_map, &snapshot2_map);
    let deleted_files = find_deleted_files(&snapshot1_map, &snapshot2_map);

    Ok(SnapshotComparison {
        new_files,
        modified_files,
        deleted_files,
    })
}



pub fn restore_snapshot(dir: &str, snapshot_id: usize, dry_run: bool) -> io::Result<RestoreReport> {
    validate_permissions(dir)?;

    let base_path = Path::new(dir);
    
    // Step 1: Load snapshots and find the target snapshot
    let all_snapshots = load_all_snapshots(dir)?;
    let snapshot = find_snapshot(&all_snapshots, snapshot_id)
        .ok_or_else(|| io::Error::new(
            ErrorKind::NotFound, 
            format!("Snapshot {} does not exist. Available snapshots: {}", 
                snapshot_id,
                all_snapshots.snapshots.iter()
                    .map(|s| s.id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        ))?;

    // Step 2: Check for uncommitted changes
    if has_uncommitted_changes(dir)? {
        return Err(io::Error::new(
            ErrorKind::Other,
            "Uncommitted changes detected. Use --force to override or take a new snapshot.",
        ));
    }

    // Step 3: Ensure sufficient disk space
    if !has_available_space(dir, snapshot)? {
        return Err(io::Error::new(
            ErrorKind::Other,
            "Insufficient disk space for restoration.",
        ));
    }

    // Step 4: Generate restore report
    let current_states = collect_file_states(dir)?;
    let current_map = create_file_map(&current_states);
    let snapshot_map = create_file_map(&snapshot.file_states);
    let report = generate_restore_report(&current_map, &snapshot_map);

    if dry_run {
        return Ok(report); // Dry run; do not apply changes
    }

    // Step 5: Execute restore operations
    perform_restore(base_path, snapshot_id, &report)?;

    Ok(report)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_initialize_metadata_directory() {
        let test_dir = tempdir().unwrap(); // Use a unique temp directory
        let test_path = test_dir.path().to_str().unwrap();

        // Initialize the directory
        initialize_timemachine(test_path).unwrap();

        // Check if metadata.json exists
        let metadata_path = Path::new(test_path)
            .join(".timemachine")
            .join("metadata.json");


        let metadata_content = fs::read_to_string(metadata_path).unwrap();
        assert_eq!(metadata_content, r#"{"snapshots":[]}"#);

        // Temporary directory is automatically cleaned up
    }

    #[test]
    fn test_take_snapshot() {
        let test_dir = tempdir().unwrap(); // Use a unique temp directory
        let test_path = test_dir.path().to_str().unwrap();

        // Initialize the directory for Time Machine
        initialize_timemachine(test_path).unwrap();

        // Create some test files
        let file1 = Path::new(test_path).join("file1.txt");
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "Hello, world!").unwrap();

        let file2 = Path::new(test_path).join("file2.txt");
        let mut f2 = File::create(&file2).unwrap();
        writeln!(f2, "Time Machine").unwrap();

        // Take a snapshot
        take_snapshot(test_path).unwrap();

        // Verify metadata.json is updated
        let metadata_path = Path::new(test_path)
            .join(".timemachine")
            .join("metadata.json");
        let metadata_content = fs::read_to_string(metadata_path).unwrap();
        let metadata: SnapshotMetadata = serde_json::from_str(&metadata_content).unwrap();

        assert_eq!(metadata.snapshots.len(), 1);
        assert_eq!(metadata.snapshots[0].changes, 2);

        // Temporary directory is automatically cleaned up
    }

    #[test]
    fn test_compare_snapshots() {
        // Create a temporary directory for testing
        let test_dir = tempdir().unwrap();
        let test_path = test_dir.path().to_str().unwrap();

        // Initialize the directory for Time Machine
        initialize_timemachine(test_path).unwrap();

        // Create some test files
        let file1 = Path::new(test_path).join("file1.txt");
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "Hello, world!").unwrap();

        let file2 = Path::new(test_path).join("file2.txt");
        let mut f2 = File::create(&file2).unwrap();
        writeln!(f2, "Time Machine").unwrap();

        // Take the first snapshot
        take_snapshot(test_path).unwrap();

        // Modify one of the files
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "Hello, updated world!").unwrap();

        // Create a new file
        let file3 = Path::new(test_path).join("file3.txt");
        let mut f3 = File::create(&file3).unwrap();
        writeln!(f3, "New file in second snapshot").unwrap();

        // Take the second snapshot
        take_snapshot(test_path).unwrap();

        // Compare the two snapshots (ID 1 and ID 2)
        let comparison = differentiate_snapshots(test_path, 1, 2).unwrap();

        // Assert that the comparison is correct
        assert_eq!(comparison.new_files, vec!["file3.txt"]);
        assert_eq!(comparison.deleted_files, Vec::<String>::new());
        assert_eq!(comparison.modified_files.len(), 1);

        // Assert detailed info for modified file (file1.txt)
        let modified_file = &comparison.modified_files[0];
        assert_eq!(modified_file.path, "file1.txt");
        assert_eq!(modified_file.old_size, 14); // Size of "Hello, world!\n"
        assert_eq!(modified_file.new_size, 22); // Size of "Hello, updated world!\n"
        assert_eq!(
            modified_file.old_hash,
            "d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5"
        );
        assert_eq!(
            modified_file.new_hash,
            "05c9a0cb7e51316bce559640f1cc42d6cf5a8e9c5c870e5f742e2533e669f73d"
        );

        // Verify timestamps are present (actual values will vary)
        assert!(!modified_file.old_last_modified.is_empty());
        assert!(!modified_file.new_last_modified.is_empty());

        // Temporary directory is automatically cleaned up
    }
}
