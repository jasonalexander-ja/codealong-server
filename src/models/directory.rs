use super::{
    file::File,
    user_activity
};

use std::collections::HashMap;

use tokio::sync::RwLock;

use serde::{Serialize, Deserialize};

use async_recursion::async_recursion;

use futures::future::join_all;
use futures::future::BoxFuture;


/// Possible errors when handling directory operations. 
#[derive(Clone, Serialize, Deserialize)]
pub enum DirError {
    /// A named directory cannot be accessed as it is write locked. 
    Locked(String),
    /// A directory of a given name cannot be found. 
    NotFound(String),
    /// A given path indexer is out of range for a given path. 
    DepthOutOfRange,
    /// A file or directory of a name already exists. 
    NameClash,
    LineLocked(user_activity::LockLine)
}

/// Serialisable responses to directory operations. 
#[derive(Clone, Serialize, Deserialize)]
pub enum DirectoryUpdated {
    ErasedDir(Vec<String>),
    CreatedDir(Vec<String>),
    RenameDir(RenameItem),
    CreatedFile(Vec<String>),
    ErasedFile(Vec<String>),
    RenameFile(RenameItem)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RenameItem {
    pub path: Vec<String>,
    pub name: String
}

/// Model of a directory that can store files and other 
/// subdirectories. 
#[derive(Default)]
pub struct Directory {
    pub files: RwLock<HashMap<String, File>>,
    pub subdirs: RwLock<HashMap<String, Directory>>
}

impl Directory {

    /// Finds a subdirectory, passing a ref to it's parent into an asynchronous cloure, 
    /// returning the result of that cloure. 
    /// 
    /// # Arguments
    /// * `path` - A slice of strings for the path. 
    /// * `level` - The index where the path should be read from. 
    /// * `closure` - An asynchronous closure that takes a reference to the 
    ///     target dir's parent, and the target directory's name. 
    /// 
    /// # Returns 
    /// * `Err(DirError::Locked(dir))` - If a given directory is 
    ///     currently write locked. 
    /// * `Err(DirError::NotFound::(dir))` - If a matching directory 
    ///     couldn't be found. 
    /// * `Err(DirError::DepthOutOfRange)` - If the given `level` 
    ///     is greater than the path length. 
    /// * `Ok(R)` - If the directory was sucessfully accessed and 
    ///     the cloure ran. 
    /// 
    #[allow(dead_code)]
    #[async_recursion]
    pub async fn transverse<F, R>(
        &self, 
        path: &[String], 
        level: usize,
        closure: F
    ) -> Result<R, DirError> 
    where 
        F: FnOnce(String, &Directory) -> BoxFuture<'_, R> + std::marker::Send + 'async_recursion 
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

    /// Finds a subdirectory, passing a ref to it's parent into an asynchronous cloure, 
    /// returning the result of that cloure, awaits for a directory to become free 
    /// if write locked. 
    /// 
    /// # Arguments
    /// * `path` - A slice of strings for the path. 
    /// * `level` - The index where the path should be read from. 
    /// * `closure` - An asynchronous closure that takes a reference to the 
    ///     target dir's parent, and the target directory's name. 
    /// 
    /// # Returns 
    /// * `Err(DirError::NotFound::(dir))` - If a matching directory 
    ///     couldn't be found. 
    /// * `Err(DirError::DepthOutOfRange)` - If the given `level` 
    ///     is greater than the path length. 
    /// * `Ok(R)` - If the directory was sucessfully accessed and 
    ///     the cloure ran. 
    /// 
    #[async_recursion]
    pub async fn transverse_blocking<F, R>(
        &self, 
        path: &[String], 
        level: usize,
        closure: F
    ) -> Result<R, DirError> 
    where 
        F: FnOnce(String, &Directory) -> BoxFuture<'_, R> + std::marker::Send + 'async_recursion 
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
        let subdirs = self.subdirs.read().await;
        let directory = match subdirs.get(dirname) {
            Some(d) => d,
            _ => return Err(DirError::NotFound(dirname.clone()))
        };
        directory.transverse_blocking(path, level + 1, closure).await
    }
    
    /// Creates a new directory with a "helloworld.txt" file. 
    pub fn new_with_file() -> Self {
        let file = File::default_with("Welcome to codealong! ");
        let files = HashMap::from([
            ("helloworld.txt".to_owned(), file)
        ]);
        
        Directory { 
            files: RwLock::new(files), 
            subdirs: RwLock::new(HashMap::new())
        }
    }

    /// Asnchronously transverses through the subdirs, reading and 
    /// copying each line of each file into a `DirectoryDTO`.
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

    /// Asynchronously reads the lines of each file, storing them into 
    /// a vector and returns a HashMap of all the files. 
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
        let file_lines = file.read().await;
        let line_futures = file_lines.iter().map(|line| async {
            line.read().await.get()
        });
        let lines = join_all(line_futures).await;
        (file_name.clone(), lines)
    }

    /// Asnchronously transverses through the subdirs, reading and 
    /// copying each line of each file into a `DirectoryDTO`.
    #[async_recursion]
    pub async fn clone_async(&self) -> Directory {
        let files = self.clone_files().await;

        let subdirs = self.clone_subdirs().await;

        Directory {
            files: RwLock::new(files),
            subdirs: RwLock::new(subdirs)
        }
    }

    async fn clone_subdirs(&self) -> HashMap<String, Directory> {
        let subdirs = self.subdirs.read().await;
        let subdir_futures = subdirs.iter()
            .map(|(name, dir)| async { (name.clone(), dir.clone_async().await) });
        let subdirs = join_all(subdir_futures).await;
        subdirs.into_iter()
            .collect()
    }

    /// Asynchronously reads the lines of each file, storing them into 
    /// a vector and returns a HashMap of all the files. 
    pub async fn clone_files(&self) -> HashMap<String, File> {
        let files = self.files.read().await;
        let file_futures = files.iter()
            .map(Directory::clone_file);
        let files = join_all(file_futures).await;
        files.into_iter()
            .collect()
    }

    pub async fn clone_file(key_vals: (&String, &File)) -> (String, File) {
        let (file_name, file) = key_vals;
        (file_name.clone(), file.clone().await)
    }
}

/// A data transfer object allowing copies of whole 
/// directories to be serialised and transmitted. 
#[derive(Clone, Serialize, Deserialize)]
pub struct DirectoryDTO {
    pub files: HashMap<String, Vec<String>>,
    pub subdirs: HashMap<String, DirectoryDTO>
}
