mod utlis;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io::{Write};
use std::path::Path;


#[derive(Serialize, Deserialize, Debug)]
struct Snapshot {
    id: usize,
    timestamp: String,
    changes: usize,
}


#[derive(Serialize, Deserialize, Debug)]
struct SnapshotMetadata {
    snapshots: Vec<Snapshot>,
}

/// Initializes the given directory for Time Machine by creating a `.timemachine` folder.
///
/// # Arguments
/// * `path` - The directory to initialize.
///
/// # Returns
/// * `Result<(), io::Error>` - An `Ok` result indicates success.
pub fn initialize_directory(path: &str) -> Result<(), std::io::Error> {
    let base_path = Path::new(path);
    let metadata_dir = base_path.join(".timemachine");

    // Create the base directory if it doesn't exist
    if !base_path.exists() {
        fs::create_dir_all(base_path)?;
    }

    if !metadata_dir.exists() {
        fs::create_dir_all(&metadata_dir)?;
    }

    // initialize metadata
    initialize_metadata(&metadata_dir)?;

    Ok(())
}

pub fn initialize_metadata(path: &Path) -> Result<(), std::io::Error> {
    let metadata_path = path.join("metadata.json");

    // check if metadata.json exists, if not create an empty one with an empty snapshot json

    if !metadata_path.exists() {
        let metadata = SnapshotMetadata {
            snapshots: Vec::new(),
        };
        let metadata_content = serde_json::to_string(&metadata)?;
        let mut file = fs::File::create(metadata_path)?;
        file.write_all(metadata_content.as_bytes())?;
    }

    Ok(())
}


pub fn take_snapshot(dir: &str) -> std::io::Result<()> {
    let base_path = Path::new(dir);
    let metadata_dir = base_path.join(".timemachine");
    let metadata_path = metadata_dir.join("metadata.json");

    // ensure that timestamp directory exisits
    if !metadata_dir.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Time machine not initialized"));
    }

    // Load existing metadata or create a new one
    let mut metadata: SnapshotMetadata = if metadata_path.exists() {
        let content = fs::read_to_string(&metadata_path)?;
        serde_json::from_str(&content)?
    } else {
        SnapshotMetadata { snapshots: Vec::new() }
    };

    // Count files in the directory (excluding the .timemachine directory)
    let files = fs::read_dir(base_path)?.filter(|entry| {
        if let Ok(entry) = entry {
            !entry.path().starts_with(&metadata_dir)
        } else {
            false
        }
    }).count();

    // Create a new snapshot entry

    let current_timestamp : String = Local::now().to_rfc3339();
    let snapshot = Snapshot {
     id:metadata.snapshots.len() +1,
        timestamp:current_timestamp,
        changes:files,
    };

    metadata.snapshots.push(snapshot);
    let updated_metadata = serde_json::to_string_pretty(&metadata)?;
    fs::write(metadata_path, updated_metadata)?;
    println!("Snapshot taken succesfully!");

    Ok(())

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
            initialize_directory(test_path).unwrap();

            // Check if metadata.json exists
            let metadata_path = Path::new(test_path)
                .join(".timemachine")
                .join("metadata.json");

            assert!(metadata_path.exists());

            let metadata_content = fs::read_to_string(metadata_path).unwrap();
            assert_eq!(metadata_content, r#"{"snapshots":[]}"#);

            // Temporary directory is automatically cleaned up
        }

        #[test]
        fn test_take_snapshot() {
            let test_dir = tempdir().unwrap(); // Use a unique temp directory
            let test_path = test_dir.path().to_str().unwrap();

            // Initialize the directory for Time Machine
            initialize_directory(test_path).unwrap();

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
            let metadata_path = Path::new(test_path).join(".timemachine").join("metadata.json");
            let metadata_content = fs::read_to_string(metadata_path).unwrap();
            let metadata: SnapshotMetadata = serde_json::from_str(&metadata_content).unwrap();

            assert_eq!(metadata.snapshots.len(), 1);
            assert_eq!(metadata.snapshots[0].changes, 2);

            // Temporary directory is automatically cleaned up
        }
    }
