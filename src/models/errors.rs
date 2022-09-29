use warp::reject::Reject;

pub trait LocalReject : Reject {}

#[derive(Debug)]
pub struct InternalServerError;
impl Reject for InternalServerError {}
impl LocalReject for InternalServerError {}

#[derive(Debug)]
pub struct NotFound;
impl Reject for NotFound {}
impl LocalReject for NotFound {}

#[derive(Debug)]
pub struct MaxCapacity;
impl Reject for MaxCapacity {}
impl LocalReject for MaxCapacity {}


#[allow(dead_code)]
#[derive(Debug)]
pub enum CodealongError {
    InternalServerError,
    NotFound,
    MaxCapacity
}
impl Reject for CodealongError {}
