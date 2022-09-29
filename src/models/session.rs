use super::user_activity::UserActivity;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};


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
}

#[allow(dead_code)]
pub struct UserState {
    pub sender: mpsc::UnboundedSender<UserActivity>,
    pub name: String
}

impl UserState {
    pub fn new(name: String, sender: mpsc::UnboundedSender<UserActivity>) -> Self {
        UserState { sender, name }
    }
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
        sender: mpsc::UnboundedSender<UserActivity>
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
