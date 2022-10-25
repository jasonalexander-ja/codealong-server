use crate::{
    models::{
        user_activity::{LockLine, CreateLine},
        session_activity::{SendTo, SessionActivity},
        session::Session, 
        server_activity::ServerActivity, 
        directory::{DirError, Directory}, file::{FileLine, FileLineLocked, FileLineAdded}
    }, endpoints::user
};

use futures::FutureExt;


pub async fn lock_line(
    user_id: &String,
    line_lock: LockLine, 
    session: &Session
) -> SendTo {
    let user_id = user_id.clone();
    let res = session.rootdir.transverse_blocking(&line_lock.filepath.clone(), 0,
        |f, d| async move { set_line_locked(f, user_id, d, line_lock).await }.boxed()).await;

    handle_locked_response(res)
}

fn handle_locked_response(res: Result<Result<FileLineLocked, DirError>, DirError>) -> SendTo {
    let lock_response = match res {
        Ok(v) => v,
        Err(e) => return wrap_dir_err(e)
    };
    match lock_response {
        Ok(v) => {
            let server = ServerActivity::LineLocked(v).wrap_to_session();
            SendTo::ToAllUsers(server)
        },
        Err(e) => wrap_dir_err(e)
    }
}

fn wrap_dir_err(e: DirError) -> SendTo {
    let serv_act = ServerActivity::DirectoryErr(e);
    let sess_act = SessionActivity::ServerActivity(serv_act);
    return SendTo::ToSameUser(sess_act);
}

async fn set_line_locked(
    filename: String, 
    user_id: String,
    dir: &Directory, 
    line_lock: LockLine
) -> Result<FileLineLocked, DirError> {
    let files = dir.files.read().await;
    let file = match files.get(&filename) {
        Some(f) => f,
        None => return Err(DirError::NotFound(filename))
    };
    let lines = file.read().await;
    let lines: Vec<&FileLine> = lines.iter().filter(|l| l.add_no == line_lock.line_no).collect();
    let line = if lines.len() == 0 { return Err(DirError::DepthOutOfRange) }
    else { lines[0] };
    let mut line_data = line.line_data.write().await;
    if let Some(_) = line_data.locked {
        return Err(DirError::LineLocked(line_lock.clone()))
    }
    line_data.locked = Some(user_id.clone());
    let res = FileLineLocked {
        add_no: line.add_no,
        user_id: user_id
    };
    Ok(res)
}


pub async fn new_line(
    user_id: &String,
    line_create: CreateLine, 
    session: &Session
) -> SendTo {
    let user_id = user_id.clone();
    let res = session.rootdir.transverse_blocking(&line_create.filepath.clone(), 0,
        |f, d| async move { 
            let files = d.files.read().await;
            let file = match files.get(&user_id) {
                Some(v) => v,
                None => return Err(DirError::NotFound(f))
            };
            
            let (new_line, _new_at) = file.insert_return_new_line(line_create.at, &user_id).await;
            Ok(new_line)
        }.boxed()
    ).await;

    handle_created_response(res)
}

fn handle_created_response(res: Result<Result<FileLineAdded, DirError>, DirError>) -> SendTo {
    let lock_response = match res {
        Ok(v) => v,
        Err(e) => return wrap_dir_err(e)
    };
    match lock_response {
        Ok(v) => {
            let server = ServerActivity::LineAdded(v).wrap_to_session();
            SendTo::ToAllUsers(server)
        },
        Err(e) => wrap_dir_err(e)
    }
}

pub async fn update_line(
    user_id: &String,
    line_create: CreateLine, 
    session: &Session
) -> SendTo {
    let user_id = user_id.clone();
    let res = session.rootdir.transverse_blocking(&line_create.filepath, 0, 
        |f, d| async move {
            let files = d.files.read().await;
            let file = match files.get(&user_id) {
                Some(v) => v,
                None => return Err(DirError::NotFound(f))
            };
            let lines = file.read().await;
            let lines: Vec<&FileLine> = lines.iter().filter(|l| l.add_no == line_create.at).collect();
            
            let (new_line, _new_at) = file.insert_return_new_line(line_create.at, &user_id).await;
            Ok(new_line)
        }.boxed()
    ).await;
    SendTo::ToNone
}

