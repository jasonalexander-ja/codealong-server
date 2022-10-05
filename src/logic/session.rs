use crate::{
    utils::settings::AppSettings,
    models::errors::CodealongError,
    models::{
        session::{SessionStore, Session},
        session_activity::SessionActivity,
        server_activity::ServerActivity
    },
    models::response::Count
};
use super::user as user_logic;

use futures::future::join_all;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc;

use warp::Reply;

use uuid::Uuid;


pub async fn sessions_capacity(
    settings: AppSettings, 
    state: SessionStore
) -> Count {
    let max_sessions = settings.max_sessions;

    let active_sessions = available_active_sessions(settings, state).await;
    let sessions: usize = active_sessions.len();

    if max_sessions > sessions {
        return Count::new(max_sessions - sessions);
    }
    return Count::new(0);
}

pub async fn available_active_sessions(
    settings: AppSettings, 
    state: SessionStore
) -> Vec<String> {
    let max_sess_users = settings.max_sess_users;
    let session = state.read().await;

    let mapped_sessions_fut = session.iter().map(|(key, value)| async {
        if value.users.read().await.len() < max_sess_users {
            return Some(key.clone());
        }
        None
    });
    let mapped_sessions = join_all(mapped_sessions_fut).await;

    mapped_sessions.into_iter().flatten().collect()
}

pub async fn make_new_session(
    user_name: String,
    ws: warp::ws::Ws, 
    settings: AppSettings, 
    sessions_str: SessionStore
) -> Result<impl Reply, CodealongError> {
    let (tx, rx) = mpsc::unbounded_channel::<SessionActivity>();

    let (session_id, user_id) = match check_add_session(settings.max_sessions, 
        user_name, 
        &sessions_str, 
        tx
    ).await {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let res_future = ws.on_upgrade(move |socket| 
        user_logic::user_thread(user_id, session_id, socket, sessions_str, rx)
    );

    Ok(res_future)
}

async fn check_add_session(
    max_sessions: usize,
    user_name: String,
    sessions_str: &SessionStore,
    tx: UnboundedSender<SessionActivity>
) -> Result<(String, String), CodealongError> {
    let mut sessions = sessions_str.write().await;

    if sessions.len() >= max_sessions {
        return Err(CodealongError::MaxCapacity)
    }

    let session_id = Uuid::new_v4().to_string();
    let user_id = Uuid::new_v4().to_string();
    let session = Session::new(user_name, user_id.clone(), tx);
    sessions.insert(session_id.clone(), session);
    Ok((session_id, user_id))
}

pub async fn stream_out_session(user_id: &String, sess_id: &String, sessions: &SessionStore) {
    let sessions = sessions.read().await;
    let session = match sessions.get(sess_id) {
        Some(val) => val,
        None => return
    };
    let users = session.users.read().await;
    let _user = match users.get(user_id) {
        Some(v) => v,
        None => return 
    };
    let _project_dir = session.rootdir.spool_to_dto().await;
    let server_act = ServerActivity::CurrentProject(_project_dir);
    let new_msg = SessionActivity::ServerActivity(server_act);
    if let Err(_err) = _user.sender.send(new_msg) {

    };
}
