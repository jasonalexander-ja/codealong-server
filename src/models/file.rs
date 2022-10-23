use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::RwLock;

use serde::{Serialize, Deserialize};

use futures::future::join_all;
use tokio::sync::RwLockReadGuard;
use tokio::sync::RwLockWriteGuard;

/// Denotes a line in a text file 
#[derive(Serialize, Deserialize, Clone)]
pub struct FileLine {
    /// The data on the line 
    pub line: String,
    /// The number of the line, in order of addition to file
    pub add_no: usize,
    /// If a line is locked to a user, the unique lock id will be stored 
    pub locked: Option<String>
}

impl FileLine {
    pub fn new(s: &str) -> Self {
        FileLine {
            line: s.to_owned(),
            add_no: 0,
            locked: None,
        }
    }
    pub fn _new_at(add_no: usize) -> Self {
        FileLine {
            line: "".to_owned(),
            add_no,
            locked: None
        }
    }
    pub fn _new_locked_at(add_no: usize, lock_id: &String) -> Self {
        FileLine {
            line: "".to_owned(),
            add_no,
            locked: Some(lock_id.clone())
        }
    }
    pub fn get(&self) -> String {
        self.line.clone()
    }
    pub fn lock(&mut self, lock_id: &String) {
        self.locked = Some(lock_id.clone());
    }
    pub fn _unlock(&mut self) {
        self.locked = None;
    }
}

impl Default for FileLine {
    fn default() -> Self {
        FileLine { line: "".to_owned(), add_no: 0, locked: None }
    }
}

pub struct FileLine

pub struct File {
    pub line_count: AtomicUsize,
    pub lines: RwLock<Vec<RwLock<FileLine>>>
}

impl File {
    pub async fn read(&self) -> RwLockReadGuard<Vec<RwLock<FileLine>>> {
        self.lines.read().await
    }

    pub async fn _write(&self) -> RwLockWriteGuard<Vec<RwLock<FileLine>>> {
        self.lines.write().await
    }
    pub fn default_with(val: &str) -> Self {
        let line = vec![RwLock::new(FileLine::new(val))];
        File {
            lines: RwLock::new(line),
            line_count: AtomicUsize::new(0)
        }
    }

    pub async fn clone(&self) -> Self {
        let file_lines = self.lines.read().await;
        let line_futures = file_lines.iter().map(|line| async { 
            RwLock::new(line.read().await.clone())
        });
        let lines = join_all(line_futures).await;
        File {
            lines: RwLock::new(lines),
            line_count: AtomicUsize::new(self.line_count.load(Ordering::Acquire))
        }
    }

    pub async fn _insert_return_new_line(&self, at: usize, user_id: &String) -> (FileLine, usize) {
        let mut lines = self._write().await;
        let len = lines.len();
        let add_no = self.line_count.fetch_add(1, Ordering::Relaxed);
        let line = FileLine::_new_locked_at(add_no, user_id);
        let line_copy = line.clone();
        let inserted_at = if at <= len {
            lines.push(RwLock::new(line));
            len
        }
        else {
            lines.insert(at, RwLock::new(line));
            at
        };
        (line_copy, inserted_at)
    }

}
