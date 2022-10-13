use crate::{
    models::{
        user_activity::LockLine,
        session_activity::{SendTo, SessionActivity},
        session::Session, server_activity::ServerActivity, directory::DirError,
        directory::File
    }
};

use futures::{stream::SplitStream, FutureExt};

use warp::ws::WebSocket;


pub async fn lock_line(
    line_lock: LockLine, 
    session: &Session, 
    ws: &mut SplitStream<WebSocket>
) -> SendTo {
    let res = session.rootdir.transverse_blocking(&line_lock.filepath.clone(), 
        0, 
        |filename, dir| async move {
        let files = dir.files.read().await;
        let file = match files.get(&filename) {
            Some(f) => f,
            None => return Err(DirError::NotFound(filename))
        };
        let locked_line = match file.get(line_lock.line) {
            Some(v) => v,
            None => return Err(DirError::LineLocked(line_lock.clone()))
        };
        let line = match locked_line.try_write() {
            Ok(v) => v,
            _ => return Err(DirError::LineLocked(line_lock.clone()))
        };
        Ok::<(), DirError>(())
    }.boxed()).await;

    handle_response(res)
}

async fn line_editing(file: &File) {

}

fn handle_response(res: Result<Result<(), DirError>, DirError>) -> SendTo {
    let sess_response = match res {
        Ok(v) => v,
        Err(e) => return wrap_dir_err(e)
    };
    match sess_response {
        Ok(v) => (),
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
