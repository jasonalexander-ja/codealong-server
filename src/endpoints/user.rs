use crate::{
    logic::user as user_logic,
    models::session::SessionStore,
    utils::settings::AppSettings,
};

use warp::Filter;
use warp::filters::BoxedFilter;
use warp::reply::Reply;
use warp::reject;
use warp::reject::Rejection;


fn join_session(
    session: &BoxedFilter<(SessionStore, )>, 
    settings: &BoxedFilter<(AppSettings, )>
) -> BoxedFilter<(impl Reply,)> {
    warp::path("join")
        .and(warp::ws())
        .and(warp::path::param())
        .and(warp::path::param())
        .and(settings.clone())
        .and(session.clone())
        .and_then(|
            ws: warp::ws::Ws, 
            session_id: String, 
            user_name: String,
            settings: AppSettings, 
            sessions_str: SessionStore
        | async move {
            match user_logic::new_user(session_id, user_name, ws, settings, sessions_str).await {
                Ok(val) => Ok::<_, Rejection>(val),
                Err(err) => Err(reject::custom(err))
            }
        })
        .boxed()
}

pub fn make_users_filters(
    session: &BoxedFilter<(SessionStore, )>, 
    settings: &BoxedFilter<(AppSettings, )>
) -> BoxedFilter<(impl Reply, )> {

    let join_session = join_session(session, settings);

    let users = join_session;

    warp::path("users")
        .and(users)
        .boxed()
}

