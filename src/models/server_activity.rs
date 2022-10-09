use super::directory::DirectoryDTO;
use super::directory::{DirError, DirectoryUpdated};

use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize)]
pub enum ServerActivity {
    CurrentProject(DirectoryDTO),
    DirectoryErr(DirError),
    DirectoryUpdate(DirectoryUpdated)
}

#[allow(dead_code)]
pub enum SendTo {
    ToSameUser(ServerActivity),
    ToOtherUsers(ServerActivity),
    ToAllUsers(ServerActivity),
    None
}
