use super::directory::DirectoryDTO;
use super::directory::{DirError, DirResponse};

use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize)]
pub enum ServerActivity {
    CurrentProject(DirectoryDTO),
    DirectoryErr(DirError),
    DirectoryUpdate(DirResponse)
}
