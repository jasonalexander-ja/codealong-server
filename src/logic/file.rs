use crate::{
    models::{
        user_activity::LockLine,
        session_activity::{SendTo, SessionActivity},
        session::Session, 
        server_activity::ServerActivity, 
        directory::{DirError, Directory}, file::FileLine
    }
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

    handle_response(res)
}

fn handle_response(res: Result<Result<FileLine, DirError>, DirError>) -> SendTo {
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
) -> Result<FileLine, DirError> {
    let files = dir.files.read().await;
    let file = match files.get(&filename) {
        Some(f) => f,
        None => return Err(DirError::NotFound(filename))
    };
    let lines = file.read().await;
    let line = match lines.get(line_lock.line_pos) {
        Some(v) => v,
        None => return Err(DirError::DepthOutOfRange)
    };
    let mut line = line.write().await;
    if let Some(_) = line.locked {
        return Err(DirError::LineLocked(line_lock.clone()))
    }
    line.lock(&user_id);
    Ok::<FileLine, DirError>(line.clone())
}


pub async fn new_line(
    at: usize,
    user_id: &String,
    line_lock: LockLine, 
    session: &Session
) -> SendTo {
    let user_id = user_id.clone();
    let res = session.rootdir.transverse_blocking(&line_lock.filepath.clone(), 0,
        |f, d| async move { 
            let files = d.files.read().await;
            let file = match files.get(&user_id) {
                Some(v) => v,
                None => return Err(DirError::NotFound(f))
            };
            
            let (new_line, _new_at) = file._insert_return_new_line(at, &user_id).await;
            Ok(new_line)
        }.boxed()
    ).await;

    handle_response(res)
}

