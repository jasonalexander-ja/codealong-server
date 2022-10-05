use super::directory::DirectoryDTO;

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum ServerActivity {
    CurrentProject(DirectoryDTO)
}

