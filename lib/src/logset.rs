use super::serde::{Serialize, Deserialize};
use super::logfile::Logfile;
use super::error::BoxResult;
use super::serde_yaml;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::str;
use std::path::Path;

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

    pub fn from_path(path: &str) -> BoxResult<Self> {
        match File::open(Path::new(&path)) {
            Ok(mut file) => Self::from_reader(&mut file),
            Err(_err) => Ok(Self::new())
        }
    }

    pub fn from_reader(reader: &mut dyn Read) -> BoxResult<Self> {
        let mut r = vec![];
        reader.read_to_end(&mut r)?;
        let s = str::from_utf8(&r)?;
        Self::deserialize(&s)
    }

    pub fn deserialize(s: &str) -> BoxResult<Self> {
        Ok(serde_yaml::from_str(s)?)
    }

    pub fn push(&mut self, logfile: Logfile) {
        self.logs.push(logfile)
    }

    pub fn pop(&mut self) -> Option<Logfile> {
        self.logs.pop()
    }

    pub fn remove(&mut self, index: usize) -> Option<Logfile> {
        if self.logs.len() > index {
            Some(self.logs.remove(index))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.logs.len()
    }

    pub fn serialize(&self) -> BoxResult<String> {
        Ok(serde_yaml::to_string(&self)?)
    }

    pub fn to_file(&self, path: &str) -> BoxResult<()> {
        let mut file = std::fs::File::create(path)?;
        file.write_all(self.serialize()?.as_bytes())?;
        Ok(())
    }
}
