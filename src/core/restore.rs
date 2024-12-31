use crate::core::models::{FileState, RestoreReport, Snapshot, SnapshotMetadata};
use crate::core::snapshot::{
    collect_file_states, find_deleted_files, find_modified_files, find_new_files,
    load_all_snapshots,
};
use serde_json;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;
use std::{fs, io};
use sysinfo::{DiskRefreshKind, Disks};

pub fn has_uncommitted_changes(dir: &str) -> io::Result<bool> {
    let current_files_state = collect_file_states(&dir)?;
    let all_snapshots = load_all_snapshots(dir)?;

    if let Some(latest_snapshot) = all_snapshots.snapshots.last() {
        Ok(latest_snapshot.file_states != current_files_state) //  true, has uncommitted changes
    } else {
        Ok(!current_files_state.is_empty()) // no snapshots, all files are uncommited changes
    }
}

pub fn has_available_space(dir: &str, snapshot: &Snapshot) -> io::Result<bool> {
    let base_dir = Path::new(&dir);
    let abs_path = base_dir.canonicalize()?;

    let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::everything());

    let available_space = disks
        .iter()
        .find(|disk| abs_path.starts_with(disk.mount_point()))
        .map(|disk| disk.available_space())
        .unwrap_or(0);

    let required_space = snapshot.file_states.iter().map(|s| s.size).sum();

    Ok(available_space >= required_space)
}

pub fn validate_permissions(dir: &str) -> io::Result<()> {
    let test_dir = Path::new(dir).join(".timemachine_test");
    fs::create_dir_all(&test_dir)?; // Create test directory if it doesn't exist

    let test_path = test_dir.join("hi.txt");
    fs::write(&test_path, "Hello World")?; // check write permissions
    fs::remove_file(&test_path)?;
    fs::remove_dir(&test_dir)?; // Clean up test directory

    Ok(())
}

pub fn generate_restore_report(
    old_snapshot: &HashMap<String, &FileState>,
    new_snapshot: &HashMap<String, &FileState>,
) -> RestoreReport {
    let added = find_new_files(new_snapshot, old_snapshot);
    let modified: Vec<String> = find_modified_files(old_snapshot, new_snapshot)
        .into_iter()
        .map(|detail| detail.path)
        .collect();
    let deleted = find_deleted_files(old_snapshot, new_snapshot);

    // Unchanged files can be derived by excluding modified and added files
    let unchanged = old_snapshot
        .keys()
        .filter(|path| {
            new_snapshot.contains_key(*path) && !added.contains(path) && !modified.contains(path)
        })
        .cloned()
        .collect();

    RestoreReport {
        added,
        modified,
        deleted,
        unchanged,
    }
}

pub fn perform_restore(
    base_path: &Path,
    snapshot_id: usize,
    report: &RestoreReport,
) -> io::Result<()> {
    // First validate that the snapshot exists in metadata
    let metadata_path = base_path.join(".timemachine").join("metadata.json");
    let metadata_content = fs::read_to_string(&metadata_path)?;
    let metadata: SnapshotMetadata = serde_json::from_str(&metadata_content)?;

    let snapshot = metadata
        .snapshots
        .iter()
        .find(|s| s.id == snapshot_id)
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::NotFound,
                format!(
                    "Snapshot {} does not exist. Available snapshots: {}",
                    snapshot_id,
                    metadata
                        .snapshots
                        .iter()
                        .map(|s| s.id.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        })?;

    // Now restore files based on the snapshot's file states
    for file_state in &snapshot.file_states {
        let target_path = base_path.join(&file_state.path);
        if report.added.contains(&file_state.path) || report.modified.contains(&file_state.path) {
            // Create parent directories if they don't exist
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Write the file content
            if let Some(content) = &file_state.content {
                fs::write(&target_path, content)?;
            } else {
                return Err(io::Error::new(
                    ErrorKind::NotFound,
                    format!("Content not available for file: {}", file_state.path),
                ));
            }
        }
    }

    // Handle deletions
    for path in &report.deleted {
        let target_path = base_path.join(path);
        if target_path.exists() {
            fs::remove_file(&target_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::take_snapshot;
    use tempfile::tempdir;

    #[test]
    fn test_has_uncommitted_changes() -> io::Result<()> {
        let test_dir = tempdir()?;
        let test_path = test_dir.path().to_str().unwrap();

        // create a new file

        let file = Path::new(test_path).join("hi.txt");
        let _f1 = fs::File::create(&file)?;

        // take snapshot
        take_snapshot(test_path)?;

        // Check for no uncommitted changes
        assert!(!has_uncommitted_changes(test_path)?); // pass when 'false'

        // Modify the file
        fs::remove_file(file)?;

        assert!(has_uncommitted_changes(test_path)?); // pass when 'true' because of new uncommitted changed

        Ok(())
    }

    #[test]
    fn test_has_available_space() -> io::Result<()> {
        use crate::core::models::FileState;
        use chrono::Local;

        let test_dir = tempdir()?;
        let test_path = test_dir.path().to_str().unwrap();

        // Create a test file with known size
        let file = Path::new(test_path).join("test.txt");
        let content = "Hello, World!"; // 13 bytes
        fs::write(&file, content)?;

        // Create a snapshot with known file size
        let file_state = FileState {
            path: "test.txt".to_string(),
            hash: "dummy_hash".to_string(),
            size: content.len() as u64,
            last_modified: "".to_string(),
            content: None,
        };

        let snapshot = Snapshot {
            id: 1,
            timestamp: Local::now().to_rfc3339(),
            changes: 1,
            file_states: vec![file_state],
        };

        // Test with actual snapshot (should have enough space for 13 bytes)
        assert!(has_available_space(test_path, &snapshot)?);

        // Create a snapshot with unreasonably large size
        let large_file_state = FileState {
            path: "test.txt".to_string(),
            hash: "dummy_hash".to_string(),
            size: 1024 * 1024 * 1024 * 1024 * 1024, // 1 PB
            last_modified: "".to_string(),
            content: Some(vec![2, 3, 5, 6, 7, 8, 9, 10]),
        };

        let large_snapshot = Snapshot {
            id: 2,
            timestamp: Local::now().to_rfc3339(),
            changes: 1,
            file_states: vec![large_file_state],
        };

        // Test with unreasonably large snapshot (should fail)
        assert!(!has_available_space(test_path, &large_snapshot)?);

        Ok(())
    }

    #[test]
    fn test_validate_permissions() -> io::Result<()> {
        use std::fs::create_dir_all;
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;

        // Test with writable directory
        let test_dir = tempdir()?;
        let test_path = test_dir.path().to_str().unwrap();
        assert!(validate_permissions(test_path).is_ok());

        // Test with read-only directory
        let readonly_dir = tempdir()?;
        let readonly_path = readonly_dir.path().to_str().unwrap();

        // Create the .timemachine_test directory first (needed for permission test)
        let test_dir_path = Path::new(readonly_path).join(".timemachine_test");
        create_dir_all(&test_dir_path)?;

        // Make directory read-only (no write permissions)
        fs::set_permissions(&test_dir_path, Permissions::from_mode(0o444))?;

        // Should fail with permission denied
        assert!(validate_permissions(readonly_path).is_err());

        // Cleanup: restore permissions to allow deletion
        fs::set_permissions(&test_dir_path, Permissions::from_mode(0o755))?;

        Ok(())
    }

    #[test]
    fn test_generate_restore_report() {
        use chrono::Local;
        use std::collections::HashMap;

        let now = Local::now().to_rfc3339();

        // Create sample file states
        let file1 = FileState {
            path: "file1.txt".to_string(),
            hash: "hash1".to_string(),
            size: 100,
            last_modified: now.clone(),
            content: None,
        };

        let file2 = FileState {
            path: "file2.txt".to_string(),
            hash: "hash2".to_string(),
            size: 200,
            last_modified: now.clone(),
            content: None,
        };

        let file2_modified = FileState {
            path: "file2.txt".to_string(),
            hash: "hash2_modified".to_string(),
            size: 250,
            last_modified: now.clone(),
            content: None,
        };

        let file3 = FileState {
            path: "file3.txt".to_string(),
            hash: "hash3".to_string(),
            size: 300,
            last_modified: now,
            content: None,
        };

        // Create old and new snapshots
        let mut old_snapshot = HashMap::new();
        old_snapshot.insert("file1.txt".to_string(), &file1);
        old_snapshot.insert("file2.txt".to_string(), &file2);

        let mut new_snapshot = HashMap::new();
        new_snapshot.insert("file2.txt".to_string(), &file2_modified);
        new_snapshot.insert("file3.txt".to_string(), &file3);

        let report = generate_restore_report(&old_snapshot, &new_snapshot);

        assert!(report.deleted.contains(&"file1.txt".to_string()));
        assert!(report.modified.contains(&"file2.txt".to_string()));
        assert!(report.added.contains(&"file3.txt".to_string()));
        assert!(report.unchanged.is_empty());
    }

    #[test]
    fn test_perform_restore_with_content() -> io::Result<()> {
        let test_dir = tempdir()?;
        let base_path = test_dir.path();

        // Simulate a snapshot
        let file1_path = base_path.join("file1.txt");
        fs::write(&file1_path, "file1 content")?;

        let metadata_dir = base_path.join(".timemachine");
        fs::create_dir_all(&metadata_dir)?;

        let metadata = SnapshotMetadata {
            snapshots: vec![Snapshot {
                id: 1,
                timestamp: "2024-12-30T00:30:24Z".to_string(),
                changes: 1,
                file_states: vec![FileState {
                    path: "file1.txt".to_string(),
                    hash: "dummy_hash".to_string(),
                    size: 13,
                    last_modified: "timestamp".to_string(),
                    content: Some(b"file1 content".to_vec()),
                }],
            }],
        };

        let metadata_path = metadata_dir.join("metadata.json");
        fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&metadata)?,
        )?;

        let report = RestoreReport {
            added: vec!["file1.txt".to_string()],
            modified: vec![],
            deleted: vec![],
            unchanged: vec![],
        };

        fs::remove_file(&file1_path)?; // Delete file to simulate restoration

        perform_restore(base_path, 1, &report)?;

        assert!(file1_path.exists());
        assert_eq!(fs::read_to_string(&file1_path)?, "file1 content");

        Ok(())
    }
}
