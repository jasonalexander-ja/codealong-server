use futures::FutureExt;
use tokio::sync::RwLock;

use crate::{
    models::{
        user_activity::{
            DirectoryUpdated, 
            RenameItem
        },
        session::{
            SessionStore, 
            Session, 
            UserState
        },
        directory::{
            DirError, 
            DirResponse, 
            Directory
        }, 
        session_activity::SessionActivity, 
        server_activity::ServerActivity,
    }
};


pub async fn directory_changed(
    user_id: &String,
    sess_id: &String, 
    dir: DirectoryUpdated, 
    sessions: &SessionStore
) {
    let sessions = sessions.read().await;
    let session = match sessions.get(sess_id) {
        Some(s) => s,
        _ => return
    };
    let users = session.users.read().await;
    let user = match users.get(user_id) {
        Some(s) => s,
        _ => return
    };
    let result = match dir {
        DirectoryUpdated::ErasedFile(v) => create_file(v, session).await,
        DirectoryUpdated::CreatedFile(v) => deleted_file(v, session).await,
        DirectoryUpdated::RenameFile(v) => rename_file(v, session).await,

        DirectoryUpdated::ErasedDir(v) => delete_dir(v, session).await,
        DirectoryUpdated::CreatedDir(v) => create_dir(v, session).await,
        DirectoryUpdated::RenameDir(v) =>  rename_dir(v, session).await,
    };
    send_response(result, user);
}

fn send_response(msg: Result<DirResponse, DirError>, user: &UserState) {
    let res = match msg {
        Ok(res) => ServerActivity::DirectoryUpdate(res),
        Err(res) => ServerActivity::DirectoryErr(res)
    };
    if let Err(_) = user.sender.send(SessionActivity::ServerActivity(res)) {
        // User has disconected, user disconect logic will run 
    };
}

async fn create_file(
    path: Vec<String>,
    session: &Session
) -> Result<DirResponse, DirError> {
    if path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    session.rootdir.transverse_blocking(&path, 0, |filename, dir| async move {
        let mut files = dir.files.write().await;
        let file = vec![RwLock::new("".to_owned())];
        if files.contains_key(&filename) {
            return Err(DirError::NameClash)
        }
        files.insert(filename.clone(), file);
        Ok(DirResponse::Created(filename))
    }.boxed()).await?
}

async fn deleted_file(
    path: Vec<String>,
    session: &Session
) -> Result<DirResponse, DirError> {
    if path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    session.rootdir.transverse_blocking(&path, 0, |filename, dir| async move {
        let mut files = dir.files.write().await;
        if !files.contains_key(&filename) {
            return Err(DirError::NotFound(filename))
        }
        files.remove(&filename);
        Ok(DirResponse::Deleted(filename))
    }.boxed()).await?
}

async fn rename_file(
    rename: RenameItem,
    session: &Session
) -> Result<DirResponse, DirError> {
    if rename.path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    session.rootdir.transverse_blocking(&rename.path.clone(), 0, |filename, dir| async move {
        let mut files = dir.files.write().await;
        let (_, target_file) = match files.get(&filename) {
            Some(v) => Directory::clone_file((&filename, v)).await,
            _ => return Err(DirError::NotFound(filename))
        };
        files.remove(&filename);
        files.insert(rename.name.clone(), target_file);
        Ok(DirResponse::Renamed(rename))
    }.boxed()).await?
}



async fn create_dir(
    path: Vec<String>,
    session: &Session
) -> Result<DirResponse, DirError> {
    if path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    session.rootdir.transverse_blocking(&path, 0, |filename, dir| async move {
        let mut dirs = dir.subdirs.write().await;
        if dirs.contains_key(&filename) {
            return Err(DirError::NameClash)
        }
        let new_dir = Directory::default();
        dirs.insert(filename.clone(), new_dir);
        Ok(DirResponse::Deleted(filename))
    }.boxed()).await?
}

async fn delete_dir(
    path: Vec<String>,
    session: &Session
) -> Result<DirResponse, DirError> {
    if path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    session.rootdir.transverse_blocking(&path, 0, |filename, dir| async move {
        let mut dirs = dir.subdirs.write().await;
        if !dirs.contains_key(&filename) {
            return Err(DirError::NotFound(filename))
        }
        dirs.remove(&filename);
        Ok(DirResponse::Deleted(filename))
    }.boxed()).await?
}

async fn rename_dir(
    rename: RenameItem,
    session: &Session
) -> Result<DirResponse, DirError> {
    if rename.path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    session.rootdir.transverse_blocking(&rename.path, 0, |filename, dir| async move {
        let mut dirs = dir.subdirs.write().await;
        let target_dir = match dirs.get(&filename) {
            Some(v) => v.clone_async().await,
            _ => return Err(DirError::NotFound(filename))
        };
        dirs.remove(&filename);
        dirs.insert(rename.name, target_dir);
        Ok(DirResponse::Deleted(filename))
    }.boxed()).await?
}
