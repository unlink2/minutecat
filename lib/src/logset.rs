use super::serde::{Serialize, Deserialize};
use super::typetag;
use super::logfile::Logfile;

#[derive(Serialize, Deserialize)]
pub struct LogSet {
    pub logs: Vec<Logfile>
}

impl LogSet {
    pub fn new() -> Self {
        Self {
            logs: vec![]
        }
    }

    pub fn push(&mut self, logfile: Logfile) {
        self.logs.push(logfile)
    }

    pub fn pop(&mut self) -> Option<Logfile> {
        self.logs.pop()
    }

    pub fn remove(&mut self, index: usize) -> Option<Logfile> {
        if self.logs.len() < index {
            Some(self.logs.remove(index))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.logs.len()
    }
}
