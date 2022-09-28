use crate::{
    utils::settings::AppSettings,
    models::session::SessionStore,
    models::response::Count
};

use futures::future::join_all;


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
