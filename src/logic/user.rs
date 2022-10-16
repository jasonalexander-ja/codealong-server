use crate::{
    models::{
        session::{
            SessionStore,
            Session,
            UserState
        },
        session_activity::SessionActivity,
        user_activity::UserActivity,
        errors::CodealongError, 
        session_activity::SendTo
    },
    utils::settings::AppSettings
};

use super::session as session_logic;
use super::directory as dir_logic;
use super::file as file_logic;

use warp::{filters::ws, ws::WebSocket};
use warp::reply::Reply;
use warp::ws::Message;

use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio_stream::wrappers::UnboundedReceiverStream;

use futures::{SinkExt, TryFutureExt, stream::SplitStream};
use futures_util::{StreamExt, stream::SplitSink};

use serde_json::{to_string as to_json_string, from_str};

use uuid::Uuid;


pub async fn new_user(
    session_id: String, 
    user_name: String,
    ws: warp::ws::Ws, 
    settings: AppSettings, 
    sessions_str: SessionStore
) -> Result<impl Reply, CodealongError> {
    let (tx, rx) = mpsc::unbounded_channel::<SessionActivity>();

    let new_user = UserState::new(user_name, tx);

    let user_id = match check_add_users(settings.max_sess_users, 
        &session_id, 
        &sessions_str, 
        new_user
    ).await {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let res_future = ws.on_upgrade(move |socket| 
        user_thread(user_id, session_id, socket, sessions_str, rx)
    );

    Ok(res_future)
}

async fn check_add_users(
    max_sess_users: usize, 
    session_id: &String, 
    sessions_str: &SessionStore,
    new_user: UserState
) -> Result<String, CodealongError> {
    let sessions = sessions_str.read().await;
    let session = match sessions.get(session_id) {
        Some(val) => val,
        _ => return Err(CodealongError::NotFound)
    };
    let mut users = session.users.write().await;

    if users.len() <= max_sess_users {
        return Err(CodealongError::MaxCapacity)
    }

    let user_id = Uuid::new_v4().to_string();

    users.insert(
        user_id.clone(), 
        new_user
    );
    Ok(user_id)
}

pub async fn user_thread(
    user_id: String,
    session_id: String,
    ws: ws::WebSocket,
    sessions: SessionStore,
    user_rx: UnboundedReceiver<SessionActivity>
) {
    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let rx = UnboundedReceiverStream::new(user_rx);

    user_send_task(rx, user_ws_tx);

    loop {
        let msg = match next_response(&mut user_ws_rx).await {
            Some(v) => v,
            None => break
        };
        process_user_resquest(
            user_id.clone(), 
            session_id.clone(), 
            msg, 
            &sessions
        ).await
    }
}

async fn next_response(user_ws_rx: &mut SplitStream<WebSocket>) -> Option<Message> {
    let msg = if let Some(m) = user_ws_rx.next().await { m }
    else { return None; };
    if let Ok(m) = msg { Some(m) }
    else { None }
}

async fn process_user_resquest(
    user_id: String, 
    sess_id: String, 
    msg: Message, 
    sessions: &SessionStore
) {
    let msg = match extract_message(&msg) {
        Some(val) => val,
        _ => return
    };
    let session = sessions.read().await;
    let session = match session.get(&sess_id) {
        Some(val) => val,
        _ => return
    };

    let res = match msg {
        UserActivity::RequestSync => 
            session_logic::stream_out_session(session).await,
        UserActivity::DirUpdated(update) => 
            dir_logic::directory_changed(update, session).await,
        UserActivity::LockLine(lock) =>
            file_logic::lock_line(&user_id, lock, session).await,
        _ => SendTo::None
    };
    send_response(&user_id, &res, session).await;
}

fn extract_message(msg: &Message) -> Option<UserActivity> {
    let msg_text = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return None
    };
    match from_str::<UserActivity>(msg_text) {
        Ok(v) => Some(v),
        Err(_) => return None
    }
}

async fn send_response(user_id: &String, res: &SendTo, session: &Session) {
    match res {
        SendTo::None => (),
        SendTo::ToAllUsers(v) => send_all_users(v, session).await,
        SendTo::ToOtherUsers(v) => send_other_users(user_id, v, session).await,
        SendTo::ToSameUser(v) => send_same_users(user_id, v, session).await
    };
}

async fn send_all_users(act: &SessionActivity, session: &Session) {
    let users = session.users.read().await;
    for (_, user) in users.iter() {
        if let Err(_) = user.sender.send(act.clone()) {
            // User has disconected, user logout code will run 
        }
    }
}

async fn send_other_users(user_id: &String, act: &SessionActivity, session: &Session) {
    let users = session.users.read().await;
    for (id, user) in users.iter() {
        if id == user_id { continue; }
        if let Err(_) = user.sender.send(act.clone()) {
            // User has disconected, user logout code will run 
        }
    }
}

async fn send_same_users(user_id: &String, act: &SessionActivity, session: &Session) {
    let users = session.users.read().await;
    if let Some(user) = users.get(user_id) {
        if let Err(_) = user.sender.send(act.clone()) {
            // User has disconected, user logout code will run 
        }
    }
}

fn user_send_task(
    rx: UnboundedReceiverStream<SessionActivity>,
    user_ws_tx: SplitSink<ws::WebSocket, Message>
) {
    let mut rx = rx;
    let mut user_ws_tx = user_ws_tx;
    tokio::task::spawn(async move {
        while let Some(_message) = rx.next().await { 
            match to_json_string(&_message) {
                Ok(string) => user_ws_tx
                    .send(Message::text(string))
                    .unwrap_or_else(|_e| { })
                    .await,
                _ => ()
            }
        }
    });
}
