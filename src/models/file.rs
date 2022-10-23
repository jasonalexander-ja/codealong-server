use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::RwLock;

use serde::{Serialize, Deserialize};

use futures::future::join_all;
use tokio::sync::RwLockReadGuard;
use tokio::sync::RwLockWriteGuard;

/// Denotes a line in a text file 
#[derive(Serialize, Deserialize, Clone)]
pub struct FileLineData {
    /// The data on the line 
    pub line: String,
    /// If a line is locked to a user, the unique lock id will be stored 
    pub locked: Option<String>
}

pub struct FileLine {
    /// The number of the line, in order of addition to file
    pub add_no: usize,
    pub line_data: RwLock<FileLineData>
}

impl FileLine {
    pub fn new(s: &str) -> Self {
        FileLine {
            line_data: RwLock::new(FileLineData {
                line: s.to_owned(),
                locked: None,
            }),
            add_no: 0,
        }
    }
    pub fn _new_at(add_no: usize) -> Self {
        FileLine {
            line_data: RwLock::new(FileLineData {
                line: "".to_owned(),
                locked: None,
            }),
            add_no
        }
    }
    pub fn _new_locked_at(add_no: usize, lock_id: &String) -> Self {
        FileLine {
            line_data: RwLock::new(FileLineData {
                line: "".to_owned(),
                locked: Some(lock_id.clone())
            }),
            add_no,
        }
    }
    pub async fn get(&self) -> String {
        self.line_data
            .read()
            .await
            .line
            .clone()
    }
}

impl Default for FileLine {
    fn default() -> Self {
        FileLine { 
            line_data: RwLock::new(FileLineData {
                line: "".to_owned(), 
                locked: None 
            }),
            add_no: 0, 
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileLineLocked {
    pub add_no: usize,
    pub user_id: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileLineAdded {
    pub add_no: usize,
    pub user_id: String
}

pub struct File {
    pub line_count: AtomicUsize,
    pub lines: RwLock<Vec<FileLine>>
}

impl File {
    pub async fn read(&self) -> RwLockReadGuard<Vec<FileLine>> {
        self.lines.read().await
    }

    pub async fn _write(&self) -> RwLockWriteGuard<Vec<FileLine>> {
        self.lines.write().await
    }
    pub fn default_with(val: &str) -> Self {
        let line = vec![FileLine::new(val)];
        File {
            lines: RwLock::new(line),
            line_count: AtomicUsize::new(0)
        }
    }

    pub async fn clone(&self) -> Self {
        let file_lines = self.lines.read().await;
        let line_futures = file_lines.iter().map(|line| async { 
            let line_data = line.line_data.read().await.clone();
            let line_data = RwLock::new(line_data);
            FileLine { line_data, add_no: line.add_no }
        });
        let lines = join_all(line_futures).await;
        File {
            lines: RwLock::new(lines),
            line_count: AtomicUsize::new(self.line_count.load(Ordering::Acquire))
        }
    }

    pub async fn insert_return_new_line(&self, at: usize, user_id: &String) -> (FileLineLocked, usize) {
        let mut lines = self._write().await;
        let len = lines.len();
        let add_no = self.line_count.fetch_add(1, Ordering::Relaxed);
        let line = FileLine {
            add_no,
            line_data: RwLock::new(FileLineData {
                line: "".to_owned(),
                locked: Some(user_id.clone())
            })
        };
        let line = FileLine::_new_locked_at(add_no, user_id);
        let line_copy = FileLineLocked {
            add_no: line.add_no,
            user_id: user_id.clone()

        };
        let inserted_at = if at <= len {
            lines.push(line);
            len
        }
        else {
            lines.insert(at, line);
            at
        };
        (line_copy, inserted_at)
    }

}
