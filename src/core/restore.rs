use crate::core::models::Snapshot;
use crate::core::snapshot::{collect_file_states, load_all_snapshots};
use std::{fs, io};
use std::path::Path;
use sysinfo::{DiskRefreshKind, Disks};

pub fn has_uncommitted_changes(dir: &str) -> io::Result<bool> {
    let base_dir = Path::new(&dir);
    let current_files_state = collect_file_states(base_dir)?;
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


pub fn validate_permissions(dir:&str) -> io::Result<()>{
    let test_dir = Path::new(dir).join(".timemachine_test");
    fs::create_dir_all(&test_dir)?;  // Create test directory if it doesn't exist
    
    let test_path = test_dir.join("hi.txt");
    fs::write(&test_path, "Hello World")?; // check write permissions
    fs::remove_file(&test_path)?;
    fs::remove_dir(&test_dir)?;  // Clean up test directory
    
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
}
