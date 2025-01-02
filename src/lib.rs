mod core;

use chrono::prelude::*;

use std::io::ErrorKind;
use std::path::Path;
use std::{fs, io};

use core::models::{Snapshot, SnapshotComparison, SnapshotMetadata, RestoreReport};
use core::snapshot::{collect_file_states, create_file_map, find_deleted_files, find_modified_files, find_new_files, find_snapshot, load_all_snapshots};
use core::restore::{validate_permissions,generate_restore_report, has_available_space, has_uncommitted_changes, perform_restore};
use sysinfo::{DiskRefreshKind, Disks};
use crate::core::content::ContentStore;
use crate::core::models::{SnapshotListInfo, StatusInfo};

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



pub fn restore_snapshot(dir: &str, snapshot_id: usize, dry_run: bool, force:bool) -> io::Result<RestoreReport> {
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

    // Step 2: Ensure sufficient disk space
    if !has_available_space(dir, snapshot)? {
        return Err(io::Error::new(
            ErrorKind::Other,
            "Insufficient disk space for restoration.",
        ));
    }

    // Step 3: Check for uncommitted changes
    if has_uncommitted_changes(dir)? {
        // if --force flag is not applied
        if !force {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Uncommitted changes detected. Take another snapshot before proceeding to restore, or use --force to override (this will automatically create a backup of your current state).",
            ));

        }
       // create a backup when using force
        eprintln!("Creating backup snapshot of current state before force restore...");
        take_snapshot(dir)?;
        eprintln!("Backup snapshot created successfully.");
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
    eprintln!("Restoring to snapshot {}...", snapshot_id);

    perform_restore(base_path, snapshot_id, &report)?;

    eprintln!("Restore completed successfully!");

    Ok(report)
}

pub fn list_snapshots(dir: &str, detailed: bool) -> io::Result<Vec<SnapshotListInfo>> {
    let metadata = load_all_snapshots(dir)?;
    
    let mut snapshot_info = Vec::new();
    for snapshot in metadata.snapshots {
        let total_size = if detailed {
            snapshot.file_states.iter().map(|s| s.size).sum()
        } else {
            0
        };
        
        snapshot_info.push(SnapshotListInfo {
            id: snapshot.id,
            timestamp: snapshot.timestamp,
            changes: snapshot.changes,
            total_size,
        });
    }
    
    Ok(snapshot_info)
}

pub fn get_status(dir: &str) -> io::Result<StatusInfo> {
    let metadata = load_all_snapshots(dir)?;
    let latest_snapshot = metadata.snapshots.last();
    let latest_snapshot_id = latest_snapshot.map(|s| s.id);
    
    let current_states = collect_file_states(dir)?;
    let mut status = StatusInfo {
        has_uncommitted_changes: false,
        modified_files: Vec::new(),
        new_files: Vec::new(),
        deleted_files: Vec::new(),
        available_space: 0,
        latest_snapshot_id,
    };
    
    // Get available space using sysinfo
    let base_dir = Path::new(dir);
    let abs_path = base_dir.canonicalize()?;
    let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::everything());
    status.available_space = disks
        .iter()
        .find(|disk| abs_path.starts_with(disk.mount_point()))
        .map(|disk| disk.available_space())
        .unwrap_or(0);
    
    if let Some(snapshot) = latest_snapshot {
        let current_map = create_file_map(&current_states);
        let snapshot_map = create_file_map(&snapshot.file_states);
        
        status.modified_files = find_modified_files(&snapshot_map, &current_map)
            .into_iter()
            .map(|m| m.path)
            .collect();
        status.new_files = find_new_files(&current_map, &snapshot_map);
        status.deleted_files = find_deleted_files(&snapshot_map, &current_map);
        
        status.has_uncommitted_changes = !status.modified_files.is_empty() 
            || !status.new_files.is_empty() 
            || !status.deleted_files.is_empty();
    }
    
    Ok(status)
}

pub fn delete_snapshot(dir: &str, snapshot_id: usize, cleanup: bool) -> io::Result<()> {
    let base_path = Path::new(dir);
    let metadata_path = base_path.join(".timemachine").join("metadata.json");
    
    // Load and update metadata
    let mut metadata: SnapshotMetadata = {
        let content = fs::read_to_string(&metadata_path)?;
        serde_json::from_str(&content)?
    };
    
    // Find and remove the snapshot
    let snapshot_index = metadata.snapshots
        .iter()
        .position(|s| s.id == snapshot_id)
        .ok_or_else(|| io::Error::new(
            ErrorKind::NotFound,
            format!("Snapshot {} not found", snapshot_id)
        ))?;
    
    metadata.snapshots.remove(snapshot_index);
    
    // Save updated metadata
    let updated_content = serde_json::to_string_pretty(&metadata)?;
    fs::write(&metadata_path, updated_content)?;
    
    // Clean up unused content if requested
    if cleanup {
        let store = ContentStore::new(base_path);
        let used_hashes: Vec<String> = metadata.snapshots
            .iter()
            .flat_map(|s| s.file_states.iter().map(|f| f.hash.clone()))
            .collect();
        store.cleanup(&used_hashes)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;
    use crate::core::content::ContentStore;
    use crate::core::models::FileState;

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

    #[test]
    fn test_perform_restore_with_content() -> io::Result<()> {
        let test_dir = tempdir()?;
        let base_path = test_dir.path();

        // Create initial file
        let file1_path = base_path.join("file1.txt");
        fs::write(&file1_path, "file1 content")?;

        // Initialize content store and store the file
        let store = ContentStore::new(base_path);
        store.init()?;
        let hash = store.store_file(&file1_path)?;

        // Create metadata with the actual hash
        let metadata = SnapshotMetadata {
            snapshots: vec![Snapshot {
                id: 1,
                timestamp: "2024-12-30T00:30:24Z".to_string(),
                changes: 1,
                file_states: vec![FileState {
                    path: "file1.txt".to_string(),
                    hash,
                    size: 12,
                    last_modified: "timestamp".to_string(),
                }],
            }],
        };

        let metadata_dir = base_path.join(".timemachine");
        fs::create_dir_all(&metadata_dir)?;
        let metadata_path = metadata_dir.join("metadata.json");
        fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&metadata)?,
        )?;

        // Delete original file
        fs::remove_file(&file1_path)?;

        let report = RestoreReport {
            added: vec!["file1.txt".to_string()],
            modified: vec![],
            deleted: vec![],
            unchanged: vec![],
        };

        perform_restore(base_path, 1, &report)?;

        assert!(file1_path.exists());
        assert_eq!(fs::read_to_string(&file1_path)?, "file1 content");

        Ok(())
    }

    #[test]
    fn test_list_snapshots() -> io::Result<()> {
        let test_dir = tempdir()?;
        let dir = test_dir.path().to_str().unwrap();

        // Initialize directory
        initialize_timemachine(dir)?;

        // Create some test files
        let file1 = Path::new(dir).join("file1.txt");
        let mut f1 = File::create(&file1)?;
        writeln!(f1, "Hello, world!")?;

        // Take first snapshot
        take_snapshot(dir)?;

        // Create another file
        let file2 = Path::new(dir).join("file2.txt");
        let mut f2 = File::create(&file2)?;
        writeln!(f2, "Second file")?;

        // Take second snapshot
        take_snapshot(dir)?;

        // Test basic listing
        let snapshots = list_snapshots(dir, false)?;
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].id, 1);
        assert_eq!(snapshots[1].id, 2);
        assert_eq!(snapshots[0].changes, 1); // First snapshot has 1 file
        assert_eq!(snapshots[1].changes, 2); // Second snapshot has 2 files
        assert_eq!(snapshots[0].total_size, 0); // Not detailed

        // Test detailed listing
        let detailed = list_snapshots(dir, true)?;
        assert!(detailed[0].total_size > 0);
        assert!(detailed[1].total_size > 0);
        assert!(detailed[1].total_size > detailed[0].total_size); // Second snapshot should have more total size

        Ok(())
    }

    #[test]
    fn test_get_status() -> io::Result<()> {
        let test_dir = tempdir()?;
        let dir = test_dir.path().to_str().unwrap();

        // Initialize directory
        initialize_timemachine(dir)?;

        // Test status with no snapshots
        let status = get_status(dir)?;
        assert!(status.available_space > 0);
        assert!(status.latest_snapshot_id.is_none());
        assert!(!status.has_uncommitted_changes);

        // Create a file and take snapshot
        let file1 = Path::new(dir).join("file1.txt");
        let mut f1 = File::create(&file1)?;
        writeln!(f1, "Initial content")?;
        take_snapshot(dir)?;

        // Test status with no changes
        let status = get_status(dir)?;
        assert_eq!(status.latest_snapshot_id, Some(1));
        assert!(!status.has_uncommitted_changes);
        assert!(status.modified_files.is_empty());
        assert!(status.new_files.is_empty());
        assert!(status.deleted_files.is_empty());

        // Modify existing file
        let mut f1 = OpenOptions::new().write(true).open(&file1)?;
        writeln!(f1, "Modified content")?;

        // Create new file
        let file2 = Path::new(dir).join("file2.txt");
        let mut f2 = File::create(&file2)?;
        writeln!(f2, "New file")?;

        // Delete first file
        fs::remove_file(&file1)?;

        // Test status with changes
        let status = get_status(dir)?;
        assert!(status.has_uncommitted_changes);
        assert_eq!(status.new_files.len(), 1);
        assert_eq!(status.deleted_files.len(), 1);
        assert!(status.new_files.contains(&"file2.txt".to_string()));
        assert!(status.deleted_files.contains(&"file1.txt".to_string()));

        Ok(())
    }

    #[test]
    fn test_delete_snapshot() -> io::Result<()> {
        let test_dir = tempdir()?;
        let dir = test_dir.path().to_str().unwrap();

        // Initialize directory
        initialize_timemachine(dir)?;

        // Create files and take snapshots
        let file1 = Path::new(dir).join("file1.txt");
        let mut f1 = File::create(&file1)?;
        writeln!(f1, "File 1")?;
        take_snapshot(dir)?;

        let file2 = Path::new(dir).join("file2.txt");
        let mut f2 = File::create(&file2)?;
        writeln!(f2, "File 2")?;
        take_snapshot(dir)?;

        // Verify initial state
        let initial_snapshots = list_snapshots(dir, false)?;
        assert_eq!(initial_snapshots.len(), 2);

        // Test deleting non-existent snapshot
        assert!(delete_snapshot(dir, 999, false).is_err());

        // Test deleting first snapshot without cleanup
        delete_snapshot(dir, 1, false)?;
        let remaining = list_snapshots(dir, false)?;
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, 2);

        // Test deleting last snapshot with cleanup
        delete_snapshot(dir, 2, true)?;
        let final_snapshots = list_snapshots(dir, false)?;
        assert_eq!(final_snapshots.len(), 0);

        Ok(())
    }
}
