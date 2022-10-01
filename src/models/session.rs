use super::user_activity;
use super::server_activity;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use serde::{Serialize, Deserialize};

use futures::future::join_all;

use async_recursion::async_recursion;


#[derive(Clone, Serialize, Deserialize)]
pub enum SessionMessage {
    UserActivity(user_activity::UserActivity),
    ServerActivity(server_activity::ServerActivity),
}

pub type File = Vec<RwLock<String>>;

#[derive(Default)]
pub struct Directory {
    pub files: RwLock<HashMap<String, File>>,
    pub subdirs: RwLock<HashMap<String, Directory>>
}

impl Directory {
    pub fn new_with_file() -> Self {
        let file = vec![
            RwLock::new("Welcome to codealong".to_owned()),
            RwLock::new("Welcome to codealong".to_owned())
        ];
        let files = HashMap::from([
            ("helloworld.txt".to_owned(), file)
        ]);
        
        Directory { 
            files: RwLock::new(files), 
            subdirs: RwLock::new(HashMap::new())
        }
    }

    #[async_recursion]
    pub async fn spool_to_dto(&self) -> DirectoryDTO {
        let files = self.spool_files().await;

        let subdirs = self.spool_subdirs().await;

        DirectoryDTO {
            files,
            subdirs
        }
    }

    async fn spool_subdirs(&self) -> HashMap<String, DirectoryDTO> {
        let subdirs = self.subdirs.read().await;
        let subdir_futures = subdirs.iter()
            .map(|(name, dir)| async { (name.clone(), dir.spool_to_dto().await) });
        let subdirs = join_all(subdir_futures).await;
        subdirs.into_iter()
            .collect()
    }

    pub async fn spool_files(&self) -> HashMap<String, Vec<String>> {
        let files = self.files.read().await;
        let file_futures = files.iter()
            .map(Directory::spool_file);
        let files = join_all(file_futures).await;
        files.into_iter()
            .collect()
    }

    async fn spool_file(key_vals: (&String, &File)) -> (String, Vec<String>) {
        let (file_name, file) = key_vals;
        let line_futures = file.iter().map(|line| async {
            line.read().await.clone()
        });
        let lines = join_all(line_futures).await;
        (file_name.clone(), lines)
    }
}

#[allow(dead_code)]
pub struct UserState {
    pub sender: mpsc::UnboundedSender<SessionMessage>,
    pub name: String
}

impl UserState {
    pub fn new(name: String, sender: mpsc::UnboundedSender<SessionMessage>) -> Self {
        UserState { sender, name }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DirectoryDTO {
    pub files: HashMap<String, Vec<String>>,
    pub subdirs: HashMap<String, DirectoryDTO>
}

#[derive(Default)]
pub struct Session {
    pub rootdir: Directory,
    pub users: RwLock<HashMap<String, UserState>>
}

impl Session {
    pub fn new(
        base_user_name: String, 
        base_user_id: String,
        sender: mpsc::UnboundedSender<SessionMessage>
    ) -> Self {
        let users = HashMap::from([
            (base_user_id, UserState::new(base_user_name, sender))
        ]);
        Session {
            rootdir: Directory::new_with_file(),
            users: RwLock::new(users)
        }
    }
}

pub type SessionStore = Arc<RwLock<HashMap<String, Session>>>;
