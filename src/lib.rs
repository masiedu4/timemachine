mod models;
mod utils;
mod core;

use chrono::prelude::*;
use models::{FileState, ModifiedFileDetail, Snapshot, SnapshotComparison, SnapshotMetadata};
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::{fs, io};
use core::snapshot::collect_file_states;

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

    let file_states = collect_file_states(&base_path)?;


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
    let base_path = Path::new(path);
    let metadata_path = base_path.join(".timemachine/metadata.json");

    // Read the metadata file and parse it
    let metadata_content = fs::read_to_string(metadata_path)?;
    let metadata: SnapshotMetadata = serde_json::from_str(&metadata_content)?;

    // Find the snapshots by ID
    let snapshot1 = metadata
        .snapshots
        .iter()
        .find(|s| s.id == snapshot_id1)
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::NotFound,
                format!("Snapshot ID not found {}", snapshot_id1),
            )
        })?;

    let snapshot2 = metadata
        .snapshots
        .iter()
        .find(|s| s.id == snapshot_id2)
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::NotFound,
                format!("Snapshot ID not found {}", snapshot_id2),
            )
        })?;

    // Create a mapping of file paths to their states for both snapshots
    let snapshot1_file_map: std::collections::HashMap<String, &FileState> = snapshot1
        .file_states
        .iter()
        .map(|fs| (fs.path.clone(), fs))
        .collect();

    let snapshot2_file_map: std::collections::HashMap<String, &FileState> = snapshot2
        .file_states
        .iter()
        .map(|fs| (fs.path.clone(), fs))
        .collect();

    let mut new_files = Vec::new();
    let mut modified_files = Vec::new();
    let mut deleted_files = Vec::new();

    // 1. Find new files in snapshot2 (files that exist in snapshot2 but not in snapshot1)
    for (path, file_state2) in &snapshot2_file_map {
        if let Some(file_state1) = snapshot1_file_map.get(path) {
            // 2. Check for modified files (same file path, but different hash or size)
            if file_state1.hash != file_state2.hash || file_state1.size != file_state2.size {
                modified_files.push(ModifiedFileDetail {
                    path: path.clone(),
                    old_size: file_state1.size,
                    new_size: file_state2.size,
                    old_hash: file_state1.hash.clone(),
                    new_hash: file_state2.hash.clone(),
                    old_last_modified: file_state1.last_modified.clone(),
                    new_last_modified: file_state2.last_modified.clone(),
                });
            }
        } else {
            // File exists in snapshot2 but not in snapshot1 (new file)
            new_files.push(path.clone());
        }
    }

    // 3. Find deleted files in snapshot1 (files that exist in snapshot1 but not in snapshot2)
    for (path, _) in &snapshot1_file_map {
        if !snapshot2_file_map.contains_key(path) {
            deleted_files.push(path.clone());
        }
    }

    // Return a SnapshotComparison struct with the results
    Ok(SnapshotComparison {
        new_files,
        modified_files,
        deleted_files,
    })
}

#[cfg(test)]
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
