use super::user_activity;
use super::server_activity;

use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize)]
pub enum SessionActivity {
    UserActivity(user_activity::UserActivity),
    ServerActivity(server_activity::ServerActivity),
}
