
extern crate serde;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Count {
    pub value: usize
}

impl Count {
    pub fn new(value: usize) -> Self {
        Count {
            value
        }
    }
}
