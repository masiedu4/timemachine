use std::collections::HashSet;
use crate::core::utils::compute_file_hash;

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use zstd::stream::{copy_decode, copy_encode};
use crate::core::models::SnapshotMetadata;

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

    /// Returns a list of content hashes that are not referenced by any snapshot
    pub fn find_orphaned_content(&self, metadata: &SnapshotMetadata) -> io::Result<Vec<String>> {
        let mut orphaned = Vec::new();
        
        // Get all content hashes currently stored
        let stored_hashes: HashSet<String> = fs::read_dir(&self.base_path)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
            
        // Get all hashes referenced by snapshots
        let used_hashes: HashSet<String> = metadata.snapshots
            .iter()
            .flat_map(|snapshot| {
                snapshot.file_states
                    .iter()
                    .map(|state| state.hash.clone())
            })
            .collect();
            
        // Find hashes that exist in storage but aren't referenced
        for hash in stored_hashes {
            if !used_hashes.contains(&hash) {
                orphaned.push(hash);
            }
        }
        
        Ok(orphaned)
    }

    /// Automatically clean up orphaned content if it exceeds a certain threshold
    pub fn auto_cleanup(&self, metadata: &SnapshotMetadata) -> io::Result<()> {
        const ORPHANED_THRESHOLD_BYTES: u64 = 100 * 1024 * 1024; // 100MB
        
        let orphaned = self.find_orphaned_content(metadata)?;
        if orphaned.is_empty() {
            return Ok(());
        }
        
        // Calculate total size of orphaned content
        let mut total_size = 0u64;
        for hash in &orphaned {
            let path = self.base_path.join(hash);
            if let Ok(metadata) = fs::metadata(path) {
                total_size += metadata.len();
            }
        }
        
        // If orphaned content exceeds threshold, clean it up
        if total_size > ORPHANED_THRESHOLD_BYTES {
            self.cleanup(&orphaned)?;
            eprintln!(
                "Cleaned up {:.2}MB of unused content", 
                total_size as f64 / (1024.0 * 1024.0)
            );
        }
        
        Ok(())
    }
    
    pub fn cleanup(&self, to_remove: &[String]) -> io::Result<()> {
        let to_remove: std::collections::HashSet<_> = to_remove.iter().collect();
        let mut cleaned_size = 0u64;

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let hash = entry.file_name().to_string_lossy().to_string();
            // Remove files that are in our to_remove list
            if to_remove.contains(&hash) {
                if let Ok(metadata) = entry.metadata() {
                    cleaned_size += metadata.len();
                }
                fs::remove_file(entry.path())?;
            }
        }

        if cleaned_size > 0 {
            eprintln!(
                "Cleaned up {:.2}MB of content", 
                cleaned_size as f64 / (1024.0 * 1024.0)
            );
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

        // Test cleanup by passing empty list (should keep all files)
        store.cleanup(&[])?;
        assert!(store.verify_content(&hash)?, "Content should still exist after cleanup with empty list");

        // Test cleanup by passing the hash (should remove the file)
        store.cleanup(&[hash.clone()])?;
        assert!(!store.verify_content(&hash)?, "Content should be removed after cleanup with its hash");

        Ok(())
    }
}