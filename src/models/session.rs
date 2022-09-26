use std::collections::HashMap;
use std::sync::Arc;

extern crate serde;

use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;


pub type File = Vec<RwLock<String>>;

#[derive(Default)]
pub struct Directory {
    pub files: RwLock<HashMap<String, File>>,
    pub subdirs: RwLock<HashMap<String, Directory>>
}

pub struct UserState {
    sender: mpsc::UnboundedSender<Message>,
    currently_editing: Option<(String, usize)>,
    name: String
}

#[derive(Default)]
pub struct Session {
    pub rootdir: Directory,
    pub users: RwLock<HashMap<String, UserState>>
}

pub type SessionStore = Arc<HashMap<String, Session>>;
