use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

type File = Vec<RwLock<String>>;

#[derive(Default)]
struct Directory {
    pub files: HashMap<String, File>,
    pub subdirs: HashMap<String, Directory>
}

#[derive(Default)]
struct Session {
    pub rootdir: Directory,
    pub users: HashMap<String, mpsc::UnboundedSender<Message>>
}

type SessionStore = Arc<HashMap<String, Session>>;
