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
    pub line: usize
}


#[derive(Serialize, Deserialize, Clone)]
pub enum UserActivity {
    DirUpdated(DirectoryUpdated),
    FileChanged(FileChanged),
    LockLine(LockLine),
    RequestSync
}
