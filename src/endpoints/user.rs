use crate::{
    models::session::SessionStore,
    utils::settings::AppSettings
};

use warp::Filter;
use warp::filters::BoxedFilter;
use warp::reply::Reply;
use warp::reject::Rejection;


fn join_session(
    session: &BoxedFilter<(SessionStore, )>, 
    settings: &BoxedFilter<(AppSettings, )>
) -> BoxedFilter<(impl Reply,)> {
    warp::path("join")
        .and(warp::ws())
        .and(warp::path::param())
        .and(settings.clone())
        .and(session.clone())
        .and_then(|ws: warp::ws::Ws, session_id: String, settings: AppSettings, sessions_str: SessionStore| async {

            let sessions = sessions_str.read().await;

            let session = match sessions.get(&session_id) {
                Some(val) => val,
                _ => return Err(warp::reject())
            };
            let users = session.users.read().await;

            if users.len() <= settings.max_sess_users {
                return Err(warp::reject())
            }

            Ok::<_, Rejection>(ws.on_upgrade(move |socket| {}))
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

