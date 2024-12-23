use crate::FileState;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn compute_hash(path: &Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).expect("File copy failed");
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn collect_file_states(base_path: &Path) -> Result<Vec<FileState>, std::io::Error> {
    let mut file_states = Vec::new();

    let metadata_dir = base_path.join(".timemachine");

    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() || path.starts_with(&metadata_dir) {
            continue;
        }

        let metadata = fs::metadata(&path)?;
        let modified_time = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let file_state = FileState {
            path: path
                .strip_prefix(base_path)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
                .to_string_lossy()
                .to_string(),
            size: metadata.len(),
            last_modified: modified_time.to_string(),
            hash: compute_hash(&path)?,
        };

        file_states.push(file_state);
    }

    Ok(file_states)
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::initialize_directory;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_compute_hash() {
        fs::write("hello.txt", "").unwrap();
        let path = Path::new("hello.txt");

        let hash = match compute_hash(&path) {
            Ok(hash) => hash,
            Err(e) => panic!("Hash did not compute {}", e),
        };

        fs::remove_file("hello.txt").unwrap();

        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_collect_file_states() {
        let test_dir = tempdir().unwrap();
        let test_path = test_dir.path().to_str().unwrap();

        initialize_directory(test_path).unwrap();

        let file1 = Path::new(test_path).join("file1.txt");
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "Hello, world!").unwrap();

        let file_states = collect_file_states(Path::new(test_path)).unwrap();
        assert_eq!(file_states.len(), 1);
        assert_eq!(file_states[0].path, "file1.txt");
    }
}
