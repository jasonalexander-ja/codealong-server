use super::directory::DirectoryDTO;
use super::directory::{DirError, DirectoryUpdated};
use super::session_activity::SessionActivity;
use super::file::{FileLineLocked, FileLineAdded};

use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize)]
pub enum ServerActivity {
    CurrentProject(DirectoryDTO),
    DirectoryErr(DirError),
    DirectoryUpdate(DirectoryUpdated),
    LineLocked(FileLineLocked),
    LineAdded(FileLineAdded)
}

impl ServerActivity {
    pub fn wrap_to_session(self) -> SessionActivity {
        SessionActivity::ServerActivity(self)
    }
}
