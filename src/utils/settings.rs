use std::env;
extern crate dotenv;
use dotenv::dotenv;


#[derive(Clone)]
pub struct AppSettings {
    pub max_sessions: usize,
    pub max_sess_users: usize,
    pub max_proj_size_kb: usize
}

impl AppSettings {
    pub fn new() -> Self {
        dotenv().ok();

        let max_sessions = match env::var("max_sessions") {
            Ok(v) => v.parse::<usize>().unwrap_or(4),
            Err(_) => 4,
        };
        let max_sess_users = match env::var("users_per_session") {
            Ok(v) => v.parse::<usize>().unwrap_or(8),
            Err(_) => 8,
        };
        let max_proj_size_kb = match env::var("max_proj_size_kb") {
            Ok(v) => v.parse::<usize>().unwrap_or(1024),
            Err(_) => 1024,
        };

        AppSettings {
            max_sessions,
            max_sess_users,
            max_proj_size_kb
        }
    }
}

