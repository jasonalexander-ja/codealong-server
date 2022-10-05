use super::session_activity::SessionActivity;
use super::directory::Directory;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};


#[allow(dead_code)]
pub struct UserState {
    pub sender: mpsc::UnboundedSender<SessionActivity>,
    pub name: String
}

impl UserState {
    pub fn new(name: String, sender: mpsc::UnboundedSender<SessionActivity>) -> Self {
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
        sender: mpsc::UnboundedSender<SessionActivity>
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
