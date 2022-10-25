use super::directory::DirectoryUpdated;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct FileChanged {
    path: Vec<String>,
    line: usize,
    old: String,
    new: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LockLine {
    pub filepath: Vec<String>,
    pub line_no: usize
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateLine {
    pub filepath: Vec<String>,
    pub at: usize
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateLine {
    pub filepath: Vec<String>,
    pub at: usize
}

#[derive(Serialize, Deserialize, Clone)]
pub enum UserActivity {
    DirUpdated(DirectoryUpdated),
    FileChanged(FileChanged),
    LockLine(LockLine),
    CreateLine(CreateLine),
    RequestSync
}
