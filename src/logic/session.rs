use crate::{
    utils::settings::AppSettings,
    models::session::SessionStore,
    models::response::Count
};

use futures::future::join_all;


pub async fn sessions_capacity(settings: AppSettings, state: SessionStore) -> Count {
    let mapped_sessions_fut = state.iter().map(|(key, value)| async {
        if value.users.read().await.len() < settings.max_sess_users {
            return Some(key.clone());
        }
        return None;
    });
    let mapped_sessions = join_all(mapped_sessions_fut).await;
    let filtered_sessions: Vec<String> = mapped_sessions.into_iter().flatten().collect();
    Count::new(filtered_sessions.len())
}

pub async fn available_active_sessions(settings: AppSettings, state: SessionStore) -> Vec<String> {
    let mapped_sessions_fut = state.iter().map(|(key, value)| async {
        if value.users.read().await.len() < settings.max_sess_users {
            return Some(key.clone());
        }
        return None;
    });
    let mapped_sessions = join_all(mapped_sessions_fut).await;
    mapped_sessions.into_iter().flatten().collect()
}
