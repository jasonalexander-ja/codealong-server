use crate::{
    models::{
        user_activity::DirectoryUpdated,
        session::SessionStore
    }
};


pub async fn directory_changed(
    sess_id: &String, 
    dir: DirectoryUpdated, 
    sessions: &SessionStore
) {
    match dir {
        DirectoryUpdated::ErasedFile(v) => create_file(sess_id, v, sessions).await,
        DirectoryUpdated::CreatedFile(v) => deleted_file(sess_id, v, sessions).await,
        DirectoryUpdated::ErasedDir(v) => delete_dir(sess_id, v, sessions).await,
        DirectoryUpdated::CreatedDir(v) => create_dir(sess_id, v, sessions).await,
    };
}

async fn create_file(
    sess_id: &String,  
    path: Vec<String>,
    sessions: &SessionStore
) {

}

async fn deleted_file(
    sess_id: &String,  
    path: Vec<String>,
    sessions: &SessionStore
) {
    
}

async fn create_dir(
    sess_id: &String,  
    path: Vec<String>,
    sessions: &SessionStore
) {
    
}

async fn delete_dir(
    sess_id: &String,  
    path: Vec<String>,
    sessions: &SessionStore
) {
    
}
