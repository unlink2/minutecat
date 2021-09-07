use crate::async_trait::async_trait;
use crate::error::Error;
use crate::serde::{Deserialize, Serialize};
use crate::typetag;
use crate::DataSource;
use std::str;

/**
 * Http data input
 */
#[derive(Clone, Serialize, Deserialize)]
pub struct HttpDataSource {
    url: String,
}

impl HttpDataSource {
    pub fn new(url: &str) -> Self {
        Self { url: url.into() }
    }
}

#[typetag::serde]
#[async_trait]
impl DataSource for HttpDataSource {
    async fn load(&mut self) -> Result<String, Error> {
        Ok(reqwest::get(&self.url).await?.text().await?)
    }
}
