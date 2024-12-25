use crate::core::models::{FileState, ModifiedFileDetail, Snapshot, SnapshotMetadata};
use crate::core::utils::compute_hash;

use std::{fs, io};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;

pub fn load_all_snapshots(path: &str) -> io::Result<SnapshotMetadata> {
    let metadata_path = Path::new(path).join(".timemachine/metadata.json");
    let metadata_content = fs::read_to_string(metadata_path)?;
    serde_json::from_str(&metadata_content).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Failed to parse metadata: {}", e),
        )
    })
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

pub fn find_snapshot(metadata: &SnapshotMetadata, snapshot_id: usize) -> Option<&Snapshot> {
    metadata.snapshots.iter().find(|s| s.id == snapshot_id)
}

pub fn create_file_map(file_states: &[FileState]) -> HashMap<String, &FileState> {
    file_states.iter().map(|fs| (fs.path.clone(), fs)).collect()
}

pub fn find_new_files(
    new_snapshot: &HashMap<String, &FileState>,
    old_snapshot: &HashMap<String, &FileState>,
) -> Vec<String> {
    new_snapshot
        .keys()
        .filter(|path| !old_snapshot.contains_key(*path))
        .cloned()
        .collect()
}

pub fn find_modified_files(
    old_snapshot: &HashMap<String, &FileState>,
    new_snapshot: &HashMap<String, &FileState>,
) -> Vec<ModifiedFileDetail> {
    new_snapshot
        .iter()
        .filter_map(|(path, new_file)| {
            if let Some(old_file) = old_snapshot.get(path) {
                if old_file.hash != new_file.hash || old_file.size != new_file.size {
                    return Some(ModifiedFileDetail {
                        path: path.clone(),
                        old_size: old_file.size,
                        new_size: new_file.size,
                        old_hash: old_file.hash.clone(),
                        new_hash: new_file.hash.clone(),
                        old_last_modified: old_file.last_modified.clone(),
                        new_last_modified: new_file.last_modified.clone(),
                    });
                }
            }
            None
        })
        .collect()
}

pub fn find_deleted_files(
    old_snapshot: &HashMap<String, &FileState>,
    new_snapshot: &HashMap<String, &FileState>,
) -> Vec<String> {
    old_snapshot
        .keys()
        .filter(|path| !new_snapshot.contains_key(*path))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initialize_timemachine;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_collect_file_states() {
        let test_dir = tempdir().unwrap();
        let test_path = test_dir.path().to_str().unwrap();

        initialize_timemachine(test_path).unwrap();

        let file1 = Path::new(test_path).join("file1.txt");
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "Hello, world!").unwrap();

        let file_states = collect_file_states(Path::new(test_path)).unwrap();
        assert_eq!(file_states.len(), 1);
        assert_eq!(file_states[0].path, "file1.txt");
    }

    #[test]
    fn test_load_snapshots() {
        let test_dir = tempdir().unwrap();
        let test_path = test_dir.path().to_str().unwrap();

        initialize_timemachine(test_path).unwrap();

        let metadata = SnapshotMetadata { snapshots: vec![] };
        let metadata_path = Path::new(test_path).join(".timemachine/metadata.json");
        fs::write(&metadata_path, serde_json::to_string(&metadata).unwrap()).unwrap();

        let result = load_all_snapshots(test_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().snapshots.len(), 0);
    }

    #[test]
    fn test_find_snapshot() {
        let metadata = SnapshotMetadata {
            snapshots: vec![
                Snapshot {
                    id: 1,
                    timestamp: "2024-01-01T12:00:00Z".to_string(),
                    changes: 0,
                    file_states: vec![],
                },
                Snapshot {
                    id: 2,
                    timestamp: "2024-01-02T12:00:00Z".to_string(),
                    changes: 0,
                    file_states: vec![],
                },
            ],
        };

        let result = find_snapshot(&metadata, 2);
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 2);

        let not_found = find_snapshot(&metadata, 3);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_create_file_map() {
        let file_states = vec![
            FileState {
                path: "file1.txt".to_string(),
                size: 100,
                hash: "hash1".to_string(),
                last_modified: "2024-01-01T12:00:00Z".to_string(),
            },
            FileState {
                path: "file2.txt".to_string(),
                size: 200,
                hash: "hash2".to_string(),
                last_modified: "2024-01-02T12:00:00Z".to_string(),
            },
        ];

        let file_map = create_file_map(&file_states);
        assert_eq!(file_map.len(), 2);
        assert!(file_map.contains_key("file1.txt"));
        assert_eq!(file_map["file1.txt"].size, 100);
    }

    #[test]
    fn test_find_new_files() {
        let old_states = vec![FileState {
            path: "file1.txt".to_string(),
            size: 100,
            hash: "hash1".to_string(),
            last_modified: "2024-01-01T12:00:00Z".to_string(),
        }];

        let new_states = vec![
            FileState {
                path: "file1.txt".to_string(),
                size: 100,
                hash: "hash1".to_string(),
                last_modified: "2024-01-01T12:00:00Z".to_string(),
            },
            FileState {
                path: "file2.txt".to_string(),
                size: 200,
                hash: "hash2".to_string(),
                last_modified: "2024-01-02T12:00:00Z".to_string(),
            },
        ];

        let old_map = create_file_map(&old_states);
        let new_map = create_file_map(&new_states);

        let new_files = find_new_files(&new_map, &old_map);
        assert_eq!(new_files, vec!["file2.txt".to_string()]);
    }

    #[test]
    fn test_find_modified_files() {
        let old_states = vec![FileState {
            path: "file1.txt".to_string(),
            size: 100,
            hash: "oldhash".to_string(),
            last_modified: "2024-01-01T12:00:00Z".to_string(),
        }];

        let new_states = vec![FileState {
            path: "file1.txt".to_string(),
            size: 100,
            hash: "newhash".to_string(),
            last_modified: "2024-01-02T12:00:00Z".to_string(),
        }];

        let old_map = create_file_map(&old_states);
        let new_map = create_file_map(&new_states);

        let modified_files = find_modified_files(&old_map, &new_map);
        assert_eq!(modified_files.len(), 1);
        assert_eq!(modified_files[0].new_hash, "newhash".to_string());
    }

    #[test]
    fn test_find_deleted_files() {
        let old_states = vec![FileState {
            path: "file1.txt".to_string(),
            size: 100,
            hash: "hash1".to_string(),
            last_modified: "2024-01-01T12:00:00Z".to_string(),
        }];

        let new_states: Vec<FileState> = vec![];

        let old_map = create_file_map(&old_states);
        let new_map = create_file_map(&new_states);

        let deleted_files = find_deleted_files(&old_map, &new_map);
        assert_eq!(deleted_files, vec!["file1.txt".to_string()]);
    }
}