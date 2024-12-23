use std::fs;
use std::path::Path;

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
        fs::create_dir_all(metadata_dir)?;
    }

    Ok(())
}

#[cfg(test)]

mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]

    fn test_initialize_directory() {
        let test_dir = "./test_dir";

        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        initialize_directory(test_dir).unwrap();

        // Check if .timemachine directory exists
        let metadata_dir = Path::new(test_dir).join(".timemachine");

        assert!(metadata_dir.exists());

        // clean up
        fs::remove_dir_all(test_dir).unwrap();
    }
}
