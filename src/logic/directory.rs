use futures::FutureExt;
use tokio::sync::RwLock;

use crate::{
    models::{
        user_activity::DirectoryUpdated,
        session::SessionStore,
        directory::DirError
    }
};


pub async fn directory_changed(
    sess_id: &String, 
    dir: DirectoryUpdated, 
    sessions: &SessionStore
) {
    match dir {
        DirectoryUpdated::ErasedFile(v) => create_file(sess_id, v, sessions).await,
        DirectoryUpdated::CreatedFile(v) => deleted_file(sess_id, v, sessions).await,
        DirectoryUpdated::ErasedDir(v) => delete_dir(sess_id, v, sessions).await,
        DirectoryUpdated::CreatedDir(v) => create_dir(sess_id, v, sessions).await,
    };
}

async fn create_file(
    sess_id: &String,  
    path: Vec<String>,
    sessions: &SessionStore
) {
    if path.len() == 0 {
        return;
    }
    let sessions = sessions.read().await;
    let session = match sessions.get(sess_id) {
        Some(s) => s,
        _ => return
    };
    let _res = session.rootdir.transverse_blocking(&path, 0, |filename, dir| async move {
        let mut files = dir.files.write().await;
        let file = vec![RwLock::new("".to_owned())];
        if files.contains_key(&filename) {
            return Err(DirError::NameClash)
        }
        files.insert(filename.clone(), file);
        Ok::<(), DirError>(())
    }.boxed()).await;
}

async fn deleted_file(
    _sess_id: &String,  
    _path: Vec<String>,
    _sessions: &SessionStore
) {
    
}

async fn create_dir(
    _sess_id: &String,  
    _path: Vec<String>,
    _sessions: &SessionStore
) {
    
}

async fn delete_dir(
    _sess_id: &String,  
    _path: Vec<String>,
    _sessions: &SessionStore
) {
    
}
