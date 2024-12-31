use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use zstd::stream::{copy_encode, copy_decode};
use sha2::{Digest, Sha256};
use crate::core::utils::compute_file_hash;

pub struct ContentStore {
    base_path: PathBuf,
}

impl ContentStore {
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.join(".timemachine").join("contents"),
        }
    }

    pub fn init(&self) -> io::Result<()> {
        fs::create_dir_all(&self.base_path)
    }

    pub fn store_file(&self, file_path: &Path) -> io::Result<String> {
        // Compute hash first
        let hash = compute_file_hash(file_path)?;

        let content_path = self.base_path.join(&hash);
        if !content_path.exists() {
            // Compress and store content
            let source = File::open(file_path)?;
            let target = File::create(&content_path)?;
            copy_encode(source, target, 3)?; // compression level 3
        }

        Ok(hash)
    }

    pub fn retrieve_file(&self, hash: &str, target_path: &Path) -> io::Result<()> {
        let content_path = self.base_path.join(hash);
        if !content_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Content not found for hash: {}", hash),
            ));
        }

        // Create parent directories if needed
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Decompress and write to target
        let source = File::open(content_path)?;
        let target = File::create(target_path)?;
        copy_decode(source, target)?;

        Ok(())
    }

    pub fn cleanup(&self, used_hashes: &[String]) -> io::Result<()> {
        let used_hashes: std::collections::HashSet<_> = used_hashes.iter().collect();

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let hash = entry.file_name().to_string_lossy().to_string();
            if !used_hashes.contains(&hash) {
                fs::remove_file(entry.path())?;
            }
        }

        Ok(())
    }

    pub fn verify_content(&self, hash: &str) -> io::Result<bool> {
        let content_path = self.base_path.join(hash);
        if !content_path.exists() {
            return Ok(false);
        }

        // Decompress and verify hash
        let mut temp = Vec::new();
        let source = File::open(content_path)?;
        let mut decoder = zstd::Decoder::new(source)?;
        decoder.read_to_end(&mut temp)?;

        let mut hasher = Sha256::new();
        hasher.update(&temp);
        let computed_hash = format!("{:x}", hasher.finalize());

        Ok(computed_hash == hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_content_store() -> io::Result<()> {
        let test_dir = tempdir()?;
        let store = ContentStore::new(test_dir.path());
        store.init()?;

        // Create a test file
        let test_file = test_dir.path().join("test.txt");
        fs::write(&test_file, b"Hello, World!")?;

        // Store the file
        let hash = store.store_file(&test_file)?;

        // Verify content exists
        assert!(store.verify_content(&hash)?);

        // Retrieve to a new location
        let restored_file = test_dir.path().join("restored.txt");
        store.retrieve_file(&hash, &restored_file)?;

        // Verify content matches
        assert_eq!(
            fs::read_to_string(&test_file)?,
            fs::read_to_string(&restored_file)?
        );

        // Test cleanup
        store.cleanup(&[hash.clone()])?;
        assert!(store.verify_content(&hash)?);

        store.cleanup(&[])?;
        assert!(!store.verify_content(&hash)?);

        Ok(())
    }
}