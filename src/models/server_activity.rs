use super::session::DirectoryDTO;

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum ServerActivity {
    CurrentProject(DirectoryDTO)
}

