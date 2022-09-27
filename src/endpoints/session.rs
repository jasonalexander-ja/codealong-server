use crate::{
    logic::session as session_logic,
    models::session::SessionStore,
    utils::settings::AppSettings
};

use warp::reply::{
    self,
    Reply,
};
use warp::reject::Rejection;


pub async fn sessions_capacity(
    settings: AppSettings, 
    state: SessionStore
) -> Result<impl Reply, Rejection> {
    let result = session_logic::sessions_capacity(settings, state).await;
    return Ok(reply::json(&result));
}

pub async fn available_active_sessions(
    settings: AppSettings, 
    state: SessionStore
) -> Result<impl Reply, Rejection> {
    let result = session_logic::available_active_sessions(settings, state).await;
    return Ok(reply::json(&result));
}

