use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;

pub fn compute_file_hash(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to open file for hashing: {}", e),
        )
    })?;

    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to read file for hashing: {}", e),
        )
    })?;

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_compute_file_hash() {
        let test_dir = tempdir().unwrap();
        let test_file = test_dir.path().join("test.txt");
        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let hash = compute_file_hash(&test_file).unwrap();
        assert_eq!(
            hash,
            "d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5"
        );
    }
}
