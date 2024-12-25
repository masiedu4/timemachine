use crate::FileState;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub fn compute_hash(path: &Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to open file '{}': {}", path.display(), e),
        )
    })?;

    let mut hasher = Sha256::new();

    io::copy(&mut file, &mut hasher).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read file for hashing '{}': {}", path.display(), e),
        )
    })?;

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}



#[cfg(test)]

mod tests {
    use super::*;

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

}
