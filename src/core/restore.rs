use crate::core::models::{FileState, RestoreReport, Snapshot, SnapshotMetadata};
use crate::core::snapshot::{
    collect_file_states, find_deleted_files, find_modified_files, find_new_files,
    load_all_snapshots,
};
use crate::core::content::ContentStore;

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
        println!("Latest snapshot: {:?}", latest_snapshot.file_states);
        println!("Current files: {:?}", current_files_state);
        Ok(latest_snapshot.file_states != current_files_state)
    } else {
        Ok(!current_files_state.is_empty())
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
    let metadata_path = base_path.join(".timemachine").join("metadata.json");
    let metadata_content = fs::read_to_string(&metadata_path)?;
    let metadata: SnapshotMetadata = serde_json::from_str(&metadata_content)?;

    let store = ContentStore::new(base_path);

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

    // Restore files
    for file_state in &snapshot.file_states {
        let target_path = base_path.join(&file_state.path);
        if report.added.contains(&file_state.path) || report.modified.contains(&file_state.path) {
            store.retrieve_file(&file_state.hash, &target_path)?;
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
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use crate::core::content::ContentStore;

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
    fn test_has_available_space() -> io::Result<()> {
        let test_dir = tempdir()?;
        let dir = test_dir.path().to_str().unwrap();
        let content = "test content";

        let file_state = FileState {
            path: "test.txt".to_string(),
            hash: "dummy_hash".to_string(),
            size: content.len() as u64,
            last_modified: "".to_string(),
        };

        let snapshot = Snapshot {
            id: 1,
            timestamp: "".to_string(),
            changes: 1,
            file_states: vec![file_state],
        };

        // Test with small file
        assert!(has_available_space(dir, &snapshot)?);

        // Test with extremely large file
        let large_file_state = FileState {
            path: "large.txt".to_string(),
            hash: "dummy_hash".to_string(),
            size: 1024 * 1024 * 1024 * 1024 * 1024, // 1 PB
            last_modified: "".to_string(),
        };

        let large_snapshot = Snapshot {
            id: 2,
            timestamp: "".to_string(),
            changes: 1,
            file_states: vec![large_file_state],
        };

        assert!(!has_available_space(dir, &large_snapshot)?);

        Ok(())
    }

    #[test]
    fn test_validate_permissions() -> io::Result<()> {
        let test_dir = tempdir()?;
        let dir = test_dir.path().to_str().unwrap();

        // Test with writable directory
        assert!(validate_permissions(dir).is_ok());

        // Note: We can't reliably test read-only scenarios in unit tests
        // as they would require root privileges

        Ok(())
    }

    #[test]
    fn test_generate_restore_report() {
        use chrono::Utc;
        let now = Utc::now().to_rfc3339();

        let file1 = FileState {
            path: "file1.txt".to_string(),
            hash: "hash1".to_string(),
            size: 100,
            last_modified: now.clone(),
        };

        let file2 = FileState {
            path: "file2.txt".to_string(),
            hash: "hash2".to_string(),
            size: 200,
            last_modified: now.clone(),
        };

        let file2_modified = FileState {
            path: "file2.txt".to_string(),
            hash: "hash2_modified".to_string(),
            size: 250,
            last_modified: now.clone(),
        };

        let file3 = FileState {
            path: "file3.txt".to_string(),
            hash: "hash3".to_string(),
            size: 300,
            last_modified: now,
        };

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
    fn test_has_uncommitted_changes() -> io::Result<()> {
        let test_dir = tempdir()?;
        let test_path = test_dir.path().to_str().unwrap();

        // Create initial file and take snapshot
        let file = test_dir.path().join("test.txt");
        fs::write(&file, "initial content")?;

        // Initialize content store
        let store = ContentStore::new(test_dir.path());
        store.init()?;
        let hash = store.store_file(&file)?;

        // Initialize .timemachine directory and metadata
        let metadata_dir = test_dir.path().join(".timemachine");
        fs::create_dir_all(&metadata_dir)?;
        
        let metadata = SnapshotMetadata {
            snapshots: vec![Snapshot {
                id: 1,
                timestamp: chrono::Utc::now().to_rfc3339(),
                changes: 1,
                file_states: vec![FileState {
                    path: "test.txt".to_string(),
                    hash: hash.clone(),
                    size: fs::metadata(&file)?.len(),
                    last_modified: fs::metadata(&file)?.modified()?.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string(),
                }],
            }],
        };

        fs::write(
            metadata_dir.join("metadata.json"),
            serde_json::to_string_pretty(&metadata)?,
        )?;

        // Initially no uncommitted changes
        let result = has_uncommitted_changes(test_path)?;
        println!("Initial check: {}", result);
        assert!(!result);

        // Modify file
        fs::write(&file, "modified content")?;

        // Now should have uncommitted changes
        let result = has_uncommitted_changes(test_path)?;
        println!("After modification: {}", result);
        assert!(result);

        Ok(())
    }
}
