use crate::async_trait::async_trait;
use crate::error::Error;
use crate::serde::{Deserialize, Serialize};
use crate::typetag;
use crate::FileDataSource;
use crate::HttpDataSource;
use crate::InMemoryDataSource;
use std::str;

#[derive(Clone, Serialize, Deserialize)]
pub enum DataSourceTypes {
    InMemory(InMemoryDataSource),
    File(FileDataSource),
    Http(HttpDataSource),
    Generic(Box<dyn DataSource>),
}

#[typetag::serde]
#[async_trait]
impl DataSource for DataSourceTypes {
    async fn load(&mut self) -> Result<String, Error> {
        match self {
            Self::InMemory(s) => s.load().await,
            Self::File(s) => s.load().await,
            Self::Http(s) => s.load().await,
            Self::Generic(s) => s.load().await,
        }
    }
}

pub trait DataSourceClone {
    fn box_clone(&self) -> Box<dyn DataSource>;
}

impl<T> DataSourceClone for T
where
    T: 'static + DataSource + Clone,
{
    fn box_clone(&self) -> Box<dyn DataSource> {
        Box::new(self.clone())
    }
}

/// A datasource knows how to fetch a logfile
/// from a location e.g. local file system,
/// ssh or http
#[typetag::serde(tag = "type")]
#[async_trait]
pub trait DataSource: DataSourceClone + Send {
    async fn load(&mut self) -> Result<String, Error>;
}

impl Clone for Box<dyn DataSource> {
    fn clone(&self) -> Box<dyn DataSource> {
        self.box_clone()
    }
}

// TODO this be tested in a sane way at all?
