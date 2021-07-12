use super::error::{BoxResult, InMemoryDataError};
use super::serde::{Serialize, Deserialize};
use super::typetag;

#[typetag::serde(tag = "type")]
pub trait DataSource {
    fn load(&mut self) -> BoxResult<String>;
}

/// this is a dummy data source
/// it provides the next item in
/// the vector until it is exhausted
/// this is mostly useful to supply dummy data
/// to logfile structs
#[derive(Serialize, Deserialize)]
pub struct InMemoryDataSource {
    data: Vec<String>
}

impl InMemoryDataSource {
    pub fn new(data: Vec<String>) -> Self {
        Self {
            data
        }
    }
}

#[typetag::serde]
impl DataSource for InMemoryDataSource {
    fn load(&mut self) -> BoxResult<String> {
        match self.data.pop() {
            Some(s) => Ok(s),
            _ => Err(Box::new(InMemoryDataError))
        }
    }
}
