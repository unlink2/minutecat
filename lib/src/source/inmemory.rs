use crate::async_trait::async_trait;
use crate::error::Error;
use crate::serde::{Deserialize, Serialize};
use crate::typetag;
use crate::DataSource;
use std::str;

/// this is a dummy data source
/// it provides the next item in
/// the vector until it is exhausted
/// this is mostly useful to supply dummy data
/// to logfile structs
#[derive(Clone, Serialize, Deserialize)]
pub struct InMemoryDataSource {
    data: Vec<String>,
}

impl InMemoryDataSource {
    pub fn new(data: Vec<String>) -> Self {
        Self { data }
    }
}

#[typetag::serde]
#[async_trait]
impl DataSource for InMemoryDataSource {
    async fn load(&mut self) -> Result<String, Error> {
        match self.data.pop() {
            Some(s) => Ok(s),
            _ => Err(Error::InMemoryDataError),
        }
    }
}
