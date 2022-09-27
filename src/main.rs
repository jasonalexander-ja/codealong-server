mod utils;
mod models;
mod endpoints;
mod logic;

extern crate serde;
extern crate futures;

use utils::settings::AppSettings;
use models::response::Count;
use models::session::{
    Session,
    SessionStore
};

use models::errors;

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::future::join_all;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::{Filter, Reply, Rejection};

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);


struct User {
    sender: mpsc::UnboundedSender<Message>,
    messages: RwLock<usize>
}

impl User {
    pub fn new(sender: mpsc::UnboundedSender<Message>) -> Self {
        User {
            sender,
            messages: RwLock::new(0)
        }
    }
}

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, User>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let app_settings = AppSettings::new();

    let session_state = SessionStore::default();

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = Users::default();
    // Turn our "state" into a new Filter...
    let users_filter = warp::any().map(move || users.clone());

    let settings_filter = warp::any().map(move || app_settings.clone());

    let session_filter = warp::any().map(move || session_state.clone());





    let available_sessions = warp::path("available_active")
        .and(warp::get())
        .and(settings_filter.clone())
        .and(session_filter.clone())
        .and_then(available_active_sessions);

    let session_capacity = warp::path("capacity")
        .and(warp::get())
        .and(settings_filter.clone())
        .and(session_filter.clone())
        .and_then(sessions_capacity);
    
    let sessions = available_sessions.or(session_capacity);

    let sessions = warp::path("session")
        .and(sessions);






    let adjust = warp::path("adjust")
        .and(warp::path::param())
        .and(warp::get())
        .and(users_filter.clone())
        .and_then(|user_id: usize, users: Users| async move {
            let users = users.read().await;
            let user = if let Some(v) = users.get(&user_id) {
                v
            } else {
                return Err(warp::reject::not_found());
            };
            let mut m = user.messages.write().await;
            *m += 1;

            Ok(warp::http::Response::new(format!("User {user_id} has sent {m} messages")))
        })
        .map(|r1: warp::http::Response<String>| r1);

    let poll = warp::path("poll")
        .and(warp::get())
        .and(users_filter.clone())
        .and_then(|users: Users| async move {
            let users = users.read().await.len();

            if users == 0 {
                return Err(warp::reject::not_found())
            }

            Ok(warp::http::Response::new(format!("Hello world {users}")))
        })
        .map(|r1: warp::http::Response<String>| r1);

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users_filter.clone())
        .and_then(|ws: warp::ws::Ws, users: Users| async move {
            // This will call our function if the handshake succeeds.
            let users_tot = users.read().await.len();

            if users_tot > 1 {
                return Err(warp::reject::not_found())
            }

            Ok(ws.on_upgrade(move |socket| user_connected(socket, users)))
        })
        .map(|v| v);

    // GET / -> index html
    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));

    let routes = index.or(chat).or(poll).or(adjust).or(sessions);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn sessions_capacity(settings: AppSettings, state: SessionStore) -> Result<impl Reply, Rejection> {
    let mapped_sessions_fut = state.iter().map(|(key, value)| async {
        if value.users.read().await.len() < settings.max_sess_users {
            return Some(key.clone());
        }
        return None;
    });
    let mapped_sessions = join_all(mapped_sessions_fut).await;
    let filtered_sessions: Vec<String> = mapped_sessions.into_iter().flatten().collect();
    Ok(warp::reply::json(&Count::new(filtered_sessions.len())))
}

async fn available_active_sessions(settings: AppSettings, state: SessionStore) -> Result<impl Reply, Rejection> {
    let mapped_sessions_fut = state.iter().map(|(key, value)| async {
        if value.users.read().await.len() < settings.max_sess_users {
            return Some(key.clone());
        }
        return None;
    });
    let mapped_sessions = join_all(mapped_sessions_fut).await;
    let filtered_sessions: Vec<String> = mapped_sessions.into_iter().flatten().collect();
    Ok(warp::reply::json(&filtered_sessions))
}

async fn user_connected(ws: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, User::new(tx));

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

async fn user_message(my_id: usize, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    let users = users.read().await;
    let usr = if let Some(s) = users.get(&my_id) {
        s
    } else {
        return;
    };

    let new_msg = format!("<User#{} Msg#{}>: {}", my_id, usr.messages.read().await, msg);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, user) in users.iter() {
        if my_id != uid {
            if let Err(_disconnected) = user.sender.send(Message::text(new_msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        } else {
            let mut v = user.messages.write().await;
            *v += 1;
            println!("User {my_id} has sent {v} messages. ");
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Warp Chat</title>
    </head>
    <body>
        <h1>Warp chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <script type="text/javascript">
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        const uri = 'ws://' + location.host + '/chat';
        const ws = new WebSocket(uri);
        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }
        ws.onopen = function() {
            chat.innerHTML = '<p><em>Connected!</em></p>';
        };
        ws.onmessage = function(msg) {
            message(msg.data);
        };
        ws.onclose = function() {
            chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
        };
        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            text.value = '';
            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;

