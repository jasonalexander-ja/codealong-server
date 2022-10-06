use std::collections::HashMap;

use tokio::sync::RwLock;

use serde::{Serialize, Deserialize};

use async_recursion::async_recursion;

use futures::future::join_all;
use futures::future::BoxFuture;


#[derive(Clone, Serialize, Deserialize)]
pub enum DirError {
    Locked(String),
    NotFound(String),
    DepthOutOfRange,
    NameClash
}

pub type File = Vec<RwLock<String>>;

#[derive(Default)]
pub struct Directory {
    pub files: RwLock<HashMap<String, File>>,
    pub subdirs: RwLock<HashMap<String, Directory>>
}

impl Directory {

    #[allow(dead_code)]
    #[async_recursion]
    pub async fn transverse<F, R>(
        &self, 
        path: &[String], 
        level: usize,
        closure: F
    ) -> Result<R, DirError> 
    where 
        F: Fn(String, &Directory) -> BoxFuture<'_, R> + std::marker::Send + 'async_recursion 
    {
        let dirname = if let Some(val) = path.get(level) {
            val
        } else { return Err(DirError::DepthOutOfRange) };

        if level + 1 >= path.len() {
            let file_dir_name = dirname.clone();
            let fut = closure(file_dir_name, self);
            let res = fut.await;
            return Ok(res)
        }
        let subdirs = match self.subdirs.try_read() {
            Ok(v) => v,
            _ => return Err(DirError::Locked(dirname.clone()))
        };
        let directory = match subdirs.get(dirname) {
            Some(d) => d,
            _ => return Err(DirError::NotFound(dirname.clone()))
        };
        directory.transverse(path, level + 1, closure).await
    }

    #[async_recursion]
    pub async fn transverse_blocking<F, R>(
        &self, 
        path: &[String], 
        level: usize,
        closure: F
    ) -> Option<R> 
    where 
        F: Fn(String, &Directory) -> BoxFuture<'_, R> + std::marker::Send + 'async_recursion 
    {
        let dirname = if let Some(val) = path.get(level) {
            val
        } else { return None };

        if level + 1 >= path.len() {
            let file_dir_name = dirname.clone();
            let fut = closure(file_dir_name, self);
            let res = fut.await;
            return Some(res)
        }
        let subdirs = self.subdirs.read().await;
        let directory = match subdirs.get(dirname) {
            Some(d) => d,
            _ => return None
        };
        directory.transverse_blocking(path, level + 1, closure).await
    }

    pub fn new_with_file() -> Self {
        let file = vec![
            RwLock::new("Welcome to codealong".to_owned()),
            RwLock::new("Welcome to codealong".to_owned())
        ];
        let files = HashMap::from([
            ("helloworld.txt".to_owned(), file)
        ]);
        
        Directory { 
            files: RwLock::new(files), 
            subdirs: RwLock::new(HashMap::new())
        }
    }

    #[async_recursion]
    pub async fn spool_to_dto(&self) -> DirectoryDTO {
        let files = self.spool_files().await;

        let subdirs = self.spool_subdirs().await;

        DirectoryDTO {
            files,
            subdirs
        }
    }

    async fn spool_subdirs(&self) -> HashMap<String, DirectoryDTO> {
        let subdirs = self.subdirs.read().await;
        let subdir_futures = subdirs.iter()
            .map(|(name, dir)| async { (name.clone(), dir.spool_to_dto().await) });
        let subdirs = join_all(subdir_futures).await;
        subdirs.into_iter()
            .collect()
    }

    pub async fn spool_files(&self) -> HashMap<String, Vec<String>> {
        let files = self.files.read().await;
        let file_futures = files.iter()
            .map(Directory::spool_file);
        let files = join_all(file_futures).await;
        files.into_iter()
            .collect()
    }

    async fn spool_file(key_vals: (&String, &File)) -> (String, Vec<String>) {
        let (file_name, file) = key_vals;
        let line_futures = file.iter().map(|line| async {
            line.read().await.clone()
        });
        let lines = join_all(line_futures).await;
        (file_name.clone(), lines)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DirectoryDTO {
    pub files: HashMap<String, Vec<String>>,
    pub subdirs: HashMap<String, DirectoryDTO>
}
