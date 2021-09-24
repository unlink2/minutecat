use super::error::Error;
use super::logfile::{EventHandler, Logfile};
use super::serde::{Deserialize, Serialize};
use super::serde_yaml;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::str;

/// A logset is the logfile manager
/// please note that logset may call blocking IO
/// and should be used in threads if blocking is not desired.
/// All structs in lib are thread safe and maye be placed inside Arc<Mutex>
/// TODO There may be an async option in the future
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct LogSet {
    pub logs: Vec<Logfile>,
}

impl LogSet {
    pub fn new() -> Self {
        Self { logs: vec![] }
    }

    pub fn from_path(path: &str) -> Result<Self, Error> {
        match File::open(Path::new(&path)) {
            Ok(mut file) => Self::from_reader(&mut file),
            Err(_err) => Ok(Self::new()),
        }
    }

    pub fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let mut r = vec![];
        reader.read_to_end(&mut r)?;
        let s = str::from_utf8(&r)?;
        Self::deserialize(s)
    }

    pub fn deserialize(s: &str) -> Result<Self, Error> {
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub async fn update(
        &mut self,
        handlers: &mut Vec<&mut dyn EventHandler>,
    ) -> Result<bool, Error> {
        for log in self.slice_mut() {
            log.update(handlers).await?;
        }
        Ok(true)
    }

    pub async fn force_update(
        &mut self,
        handlers: &mut Vec<&mut dyn EventHandler>,
    ) -> Result<bool, Error> {
        for log in self.slice_mut() {
            log.force_update(handlers).await?;
        }
        Ok(true)
    }

    pub fn slice_mut(&mut self) -> &mut [Logfile] {
        &mut self.logs[..]
    }

    pub fn serialize(&self) -> Result<String, Error> {
        Ok(serde_yaml::to_string(&self)?)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut file = std::fs::File::create(path)?;
        file.write_all(self.serialize()?.as_bytes())?;
        Ok(())
    }
}
