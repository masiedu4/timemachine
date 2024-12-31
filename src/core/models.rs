use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FileState {
    pub path: String,
    pub size: u64,
    pub last_modified: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Snapshot {
    pub id: usize,
    pub timestamp: String,
    pub changes: usize,
    pub file_states: Vec<FileState>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SnapshotMetadata {
    pub snapshots: Vec<Snapshot>,
}

#[derive(Debug)]
pub struct SnapshotComparison {
    pub new_files: Vec<String>,
    pub modified_files: Vec<ModifiedFileDetail>,
    pub deleted_files: Vec<String>,
}

#[derive(Debug)]
pub struct ModifiedFileDetail {
    pub path: String,
    pub old_size: u64,
    pub new_size: u64,
    pub old_hash: String,
    pub new_hash: String,
    pub old_last_modified: String,
    pub new_last_modified: String,
}

pub struct RestoreReport {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
    pub unchanged: Vec<String>,
}


pub struct SnapshotListInfo {
    pub id: usize,
    pub timestamp: String,
    pub changes: usize,
    pub total_size: u64,
}

pub struct StatusInfo {
    pub has_uncommitted_changes: bool,
    pub modified_files: Vec<String>,
    pub new_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub available_space: u64,
    pub latest_snapshot_id: Option<usize>,
}