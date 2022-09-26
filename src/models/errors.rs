use warp::reject::Reject;


#[derive(Debug)]
pub struct InternalServerError;

impl Reject for InternalServerError {}
