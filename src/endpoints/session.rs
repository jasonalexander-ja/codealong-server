use crate::{
    logic::session as session_logic,
    models::session::SessionStore,
    utils::settings::AppSettings
};

use warp::Filter;
use warp::filters::BoxedFilter;
use warp::reply::{self, Reply};
use warp::reject;
use warp::reject::Rejection;


fn capacity_filters(
    session: &BoxedFilter<(SessionStore, )>, 
    settings: &BoxedFilter<(AppSettings, )>
) -> BoxedFilter<(impl Reply,)> {
    warp::path("capacity")
        .and(warp::get())
        .and(settings.clone())
        .and(session.clone())
        .and_then(|settings, state| async {
            let result = session_logic::sessions_capacity(settings, state).await;
            Ok::<_, Rejection>(reply::json(&result))
        })
        .boxed()
}

fn available_filters(
    session: &BoxedFilter<(SessionStore, )>, 
    settings: &BoxedFilter<(AppSettings, )>
) -> BoxedFilter<(impl Reply, )> {
    warp::path("available_active")
        .and(warp::get())
        .and(settings.clone())
        .and(session.clone())
        .and_then(|settings, state| async {
            let result = session_logic::available_active_sessions(settings, state).await;
            Ok::<_, Rejection>(reply::json(&result))
        })
        .boxed()
}

fn make_new_session(
    session: &BoxedFilter<(SessionStore, )>, 
    settings: &BoxedFilter<(AppSettings, )>
) -> BoxedFilter<(impl Reply, )> {
    warp::path("new")
        .and(warp::ws())
        .and(warp::path::param())
        .and(settings.clone())
        .and(session.clone())
        .and_then(|
            ws: warp::ws::Ws,
            user_name: String,
            settings: AppSettings, 
            sessions_str: SessionStore
        | async move {
            match session_logic::make_new_session(user_name, ws, settings, sessions_str).await {
                Ok(val) => Ok::<_, Rejection>(val),
                Err(err) => Err(reject::custom(err))
            }
        })
        .boxed()
}

pub fn make_session_filters(
    session: &BoxedFilter<(SessionStore, )>, 
    settings: &BoxedFilter<(AppSettings, )>
) -> BoxedFilter<(impl Reply, )> {

    let available_sessions = available_filters(session, settings);
    let session_capacity = capacity_filters(session, settings);
    let new_session = make_new_session(session, settings);
    
    let sessions = available_sessions
        .or(session_capacity)
        .or(new_session);

    warp::path("session")
        .and(sessions)
        .boxed()
}

