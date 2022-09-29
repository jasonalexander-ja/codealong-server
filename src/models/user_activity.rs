use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum DirectoryUpdated {
    ErasedDir(Vec<String>),
    CreatedDir(Vec<String>),
    CreatedFile(Vec<String>),
    ErasedFile(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RequestLineEdit {
    path: Vec<String>,
    lines: Vec<usize>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileChanged {
    path: Vec<String>,
    line: usize
}

#[derive(Serialize, Deserialize, Clone)]
pub enum UserActivity {
    DirUpdated(DirectoryUpdated),
    RequestLineEdit(RequestLineEdit),
    FileChanged(FileChanged)
}