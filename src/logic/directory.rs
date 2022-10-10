use futures::FutureExt;
use tokio::sync::RwLock;

use crate::{
    models::{
        session::{
            SessionStore, 
            Session, 
        },
        directory::{
            DirError, 
            DirectoryUpdated, 
            Directory,
            RenameItem
        }, 
        server_activity::ServerActivity,
        session_activity::{SendTo, SessionActivity}
    }
};


pub async fn directory_changed(
    dir: DirectoryUpdated, 
    session: &Session
) -> SendTo {
    let server_act = match inner(session, dir).await {
        Ok(v) => ServerActivity::DirectoryUpdate(v),
        Err(v) => ServerActivity::DirectoryErr(v)
    };
    let sess_act = SessionActivity::ServerActivity(server_act);
    SendTo::ToOtherUsers(sess_act)
}


pub async fn inner(
    session: &Session, 
    dir: DirectoryUpdated
) -> Result<DirectoryUpdated, DirError> {
    match dir {
        DirectoryUpdated::ErasedFile(v) => create_file(v, session).await,
        DirectoryUpdated::CreatedFile(v) => deleted_file(v, session).await,
        DirectoryUpdated::RenameFile(v) => rename_file(v, session).await,

        DirectoryUpdated::ErasedDir(v) => delete_dir(v, session).await,
        DirectoryUpdated::CreatedDir(v) => create_dir(v, session).await,
        DirectoryUpdated::RenameDir(v) =>  rename_dir(v, session).await,
    }
}

async fn create_file(
    path: Vec<String>,
    session: &Session
) -> Result<DirectoryUpdated, DirError> {
    if path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    let path_cpy = path.iter().map(|val| val.clone()).collect();
    session.rootdir.transverse_blocking(&path, 0, |filename, dir| async move {
        let mut files = dir.files.write().await;
        let file = vec![RwLock::new("".to_owned())];
        if files.contains_key(&filename) {
            return Err(DirError::NameClash)
        }
        files.insert(filename.clone(), file);
        Ok(DirectoryUpdated::CreatedFile(path_cpy))
    }.boxed()).await?
}

async fn deleted_file(
    path: Vec<String>,
    session: &Session
) -> Result<DirectoryUpdated, DirError> {
    if path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    let path_cpy = path.iter().map(|val| val.clone()).collect();
    session.rootdir.transverse_blocking(&path, 0, |filename, dir| async move {
        let mut files = dir.files.write().await;
        if !files.contains_key(&filename) {
            return Err(DirError::NotFound(filename))
        }
        files.remove(&filename);
        Ok(DirectoryUpdated::ErasedFile(path_cpy))
    }.boxed()).await?
}

async fn rename_file(
    rename: RenameItem,
    session: &Session
) -> Result<DirectoryUpdated, DirError> {
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
        Ok(DirectoryUpdated::RenameFile(rename))
    }.boxed()).await?
}



async fn create_dir(
    path: Vec<String>,
    session: &Session
) -> Result<DirectoryUpdated, DirError> {
    if path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    let path_cpy = path.iter().map(|val| val.clone()).collect();
    session.rootdir.transverse_blocking(&path, 0, |filename, dir| async move {
        let mut dirs = dir.subdirs.write().await;
        if dirs.contains_key(&filename) {
            return Err(DirError::NameClash)
        }
        let new_dir = Directory::default();
        dirs.insert(filename.clone(), new_dir);
        Ok(DirectoryUpdated::CreatedDir(path_cpy))
    }.boxed()).await?
}

async fn delete_dir(
    path: Vec<String>,
    session: &Session
) -> Result<DirectoryUpdated, DirError> {
    if path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    let path_cpy = path.iter().map(|val| val.clone()).collect();
    session.rootdir.transverse_blocking(&path, 0, |filename, dir| async move {
        let mut dirs = dir.subdirs.write().await;
        if !dirs.contains_key(&filename) {
            return Err(DirError::NotFound(filename))
        }
        dirs.remove(&filename);
        Ok(DirectoryUpdated::ErasedDir(path_cpy))
    }.boxed()).await?
}

async fn rename_dir(
    rename: RenameItem,
    session: &Session
) -> Result<DirectoryUpdated, DirError> {
    if rename.path.len() == 0 {
        return Err(DirError::NotFound("".to_owned()));
    }
    session.rootdir.transverse_blocking(&rename.path.clone(), 0, |filename, dir| async move {
        let mut dirs = dir.subdirs.write().await;
        let target_dir = match dirs.get(&filename) {
            Some(v) => v.clone_async().await,
            _ => return Err(DirError::NotFound(filename))
        };
        dirs.remove(&filename);
        dirs.insert(rename.name.clone(), target_dir);
        Ok(DirectoryUpdated::RenameDir(rename))
    }.boxed()).await?
}
