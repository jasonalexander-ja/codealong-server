use crate::{
    models::session::SessionStore,
    models::session::Session,
    models::session::UserState,
    models::user_activity::UserActivity,
    models::errors::CodealongError,
    utils::settings::AppSettings
};

use warp::filters::ws;
use warp::reply::Reply;
use warp::ws::Message;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use futures::{SinkExt, TryFutureExt};
use futures_util::StreamExt;
use futures_util::stream::SplitSink;

use serde_json::to_string as to_json_string;
use serde_json::from_str;

use uuid::Uuid;


pub async fn new_user(
    session_id: String, 
    user_name: String,
    ws: warp::ws::Ws, 
    settings: AppSettings, 
    sessions_str: SessionStore
) -> Result<impl Reply, CodealongError> {
    let (tx, rx) = mpsc::unbounded_channel::<UserActivity>();

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
    user_rx: UnboundedReceiver<UserActivity>
) {
    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let rx = UnboundedReceiverStream::new(user_rx);

    user_send_task(rx, user_ws_tx);

    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(_e) => {
                break;
            }
        };
        process_user_response(user_id.clone(), session_id.clone(), msg, &sessions).await;
    }
}

async fn process_user_response(user_id: String, sess_id: String, msg: Message, sessions: &SessionStore) {
    let msg = match extract_message(&msg).await {
        Some(val) => val,
        _ => return
    };
    let session = sessions.read().await;
    let session = match session.get(&sess_id) {
        Some(val) => val,
        _ => return
    };
    send_user_data(session, &user_id, &msg).await;
}

async fn extract_message(msg: &Message) -> Option<UserActivity> {
    let msg_text = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return None
    };
    match from_str::<UserActivity>(msg_text) {
        Ok(v) => Some(v),
        Err(_) => return None
    }
}

async fn send_user_data(session: &Session, user_id: &String, msg: &UserActivity) {
    for (uid, user) in session.users.read().await.iter() {
        if uid == user_id {
            continue;
        }
        if let Err(_err) = user.sender.send(msg.clone()) {
            // The tx is disconected since the user thread has exited 
            // this will only happen when the user disconects 
            // which will be handled 
        };
    }
}

fn user_send_task(
    rx: UnboundedReceiverStream<UserActivity>,
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