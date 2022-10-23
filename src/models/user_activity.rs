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
    pub line_pos: usize,
    pub line_no: usize
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LineLocked {
    pub filepath: Vec<String>,
    pub line: usize,
    pub line_edit_id: String,
    pub user_id: String
}


#[derive(Serialize, Deserialize, Clone)]
pub enum UserActivity {
    DirUpdated(DirectoryUpdated),
    FileChanged(FileChanged),
    LockLine(LockLine),
    RequestSync
}
