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
    #[serde(skip)]
    client: Option<reqwest::Client>,
}

impl HttpDataSource {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.into(),
            client: None,
        }
    }
}

#[typetag::serde]
#[async_trait]
impl DataSource for HttpDataSource {
    async fn load(&mut self) -> Result<String, Error> {
        match &mut self.client {
            Some(client) => Ok(client.get(&self.url).send().await?.text().await?),
            None => {
                // cache the client and call again
                self.client = Some(reqwest::Client::builder().build()?);
                self.load().await
            }
        }
    }
}
