use crate::{
    models::{
        user_activity::LockLine,
        session_activity::{SendTo, SessionActivity},
        session::Session, 
        server_activity::ServerActivity, 
        directory::{DirError, FileLine, Directory}
    }
};

use futures::FutureExt;

use uuid::Uuid;



pub async fn lock_line(
    user_id: &String,
    line_lock: LockLine, 
    session: &Session
) -> SendTo {
    let res = session.rootdir.transverse_blocking(&line_lock.filepath.clone(), 0,
        |f, d| async move { set_line_locked(f, d, line_lock).await }.boxed()).await;

    let users = session.users.read().await;
    let user = match users.get(user_id) {
        Some(u) => u,
        None => return SendTo::None
    };

    //user.sender.send(message)

    handle_response(res)
}

fn handle_response(res: Result<Result<(), DirError>, DirError>) -> SendTo {
    let sess_response = match res {
        Ok(v) => v,
        Err(e) => return wrap_dir_err(e)
    };
    match sess_response {
        Ok(_) => (),
        Err(e) => return wrap_dir_err(e)
    };
    if let Err(e) = sess_response {
        return wrap_dir_err(e);
    };

    SendTo::None
}

fn wrap_dir_err(e: DirError) -> SendTo {
    let serv_act = ServerActivity::DirectoryErr(e);
    let sess_act = SessionActivity::ServerActivity(serv_act);
    return SendTo::ToSameUser(sess_act);
}

async fn set_line_locked(
    filename: String, 
    dir: &Directory, 
    line_lock: LockLine
) -> Result<(), DirError> {
    let files = dir.files.read().await;
    let file = match files.get(&filename) {
        Some(f) => f,
        None => return Err(DirError::NotFound(filename))
    };
    let lines = file.read().await;
    let line = match lines.get(line_lock.line) {
        Some(v) => v,
        None => return Err(DirError::DepthOutOfRange)
    };
    let mut line = line.write().await;
    if let Some(_) = line.locked {
        return Err(DirError::LineLocked(line_lock.clone()))
    }
    let edit_id = Uuid::new_v4().to_string();
    line.lock(&edit_id);
    Ok::<(), DirError>(())
}
