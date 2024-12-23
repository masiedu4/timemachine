use std::fs;
use std::io::Write;
use std::path::Path;
use serde_json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Debug)]
struct SnapshotMetadata {
    snapshots:Vec<Snapshot>
}

#[derive(Serialize, Deserialize, Debug)]
struct Snapshot {
    id:usize,
    timestamp:String,
    changes:usize,
}
/// Initializes the given directory for Time Machine by creating a `.timemachine` folder.
///
/// # Arguments
/// * `path` - The directory to initialize.
///
/// # Returns
/// * `Result<(), io::Error>` - An `Ok` result indicates success.
pub fn initialize_directory(path: &str) -> Result<(), std::io::Error> {
    let metadata_dir = Path::new(path).join(".timemachine");

    if !metadata_dir.exists() {
        fs::create_dir_all(&metadata_dir)?;
    }

    // initialize metadata
    initialize_metadata(&metadata_dir)?;

    Ok(())
}

pub fn initialize_metadata(path:&Path) -> Result<(), std::io::Error> {
    let metadata_path = path.join("metadata.json");

    // check if metadata.json exists, if not create an empty one with an empty snapshot json

    if !metadata_path.exists() {
        let metadata = SnapshotMetadata {snapshots:Vec::new()};
        let metadata_content = serde_json::to_string(&metadata)?;
        let mut file = fs::File::create(metadata_path)?;
        file.write_all(metadata_content.as_bytes())?;
    }

    Ok(())

}

#[cfg(test)]

mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]

    fn test_initialize_metadata_directory()  {
        let test_dir = "./test_dir";

        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        initialize_directory(test_dir).unwrap();

        // Check if metadata.json exists
        let metadata_path = Path::new(test_dir).join(".timemachine").join("metadata.json");

        assert!(metadata_path.exists());

        let metadata_content = fs::read_to_string(metadata_path).unwrap();
        assert_eq!(metadata_content, r#"{"snapshots":[]}"#);

        // clean up
        fs::remove_dir_all(test_dir).unwrap();
    }
}
