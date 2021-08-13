use super::async_trait::async_trait;
use super::error::{BoxResult, InMemoryDataError};
use super::serde::{Deserialize, Serialize};
use super::typetag;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::SeekFrom;
use std::path::Path;
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
    async fn load(&mut self) -> BoxResult<String> {
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
    async fn load(&mut self) -> BoxResult<String>;
}

impl Clone for Box<dyn DataSource> {
    fn clone(&self) -> Box<dyn DataSource> {
        self.box_clone()
    }
}

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
    async fn load(&mut self) -> BoxResult<String> {
        match self.data.pop() {
            Some(s) => Ok(s),
            _ => Err(Box::new(InMemoryDataError)),
        }
    }
}

/**
 * File data input
 */

/**
 * Takes an input buffer and reads the bottom n lines backwards
 */
pub struct TailReader<T>
where
    T: Read + Seek,
{
    input: T,
    line_limit: usize,
    chunk_size: u64,
}

impl<T> TailReader<T>
where
    T: Read + Seek,
{
    pub fn new(input: T, line_limit: usize) -> Self {
        Self {
            input,
            line_limit,
            chunk_size: 64,
        }
    }

    fn read_chunk(&mut self, chunk_size: usize) -> BoxResult<(usize, String)> {
        let mut buf = vec![0u8; chunk_size];
        self.input.read_exact(&mut buf)?;

        let result = str::from_utf8(&buf)?;

        Ok((self.count_lines(&result), result.into()))
    }

    fn count_lines(&self, strbuf: &str) -> usize {
        strbuf.matches("\n").count()
    }

    fn trim(&self, strbuf: &str) -> Option<usize> {
        strbuf.find("\n")
    }

    pub fn read_lines(&mut self) -> BoxResult<String> {
        // seek backwards in chunk increments
        // until enough new line characters have been found

        // get total file size
        let mut seek_pos = self.input.seek(SeekFrom::End(0))?;

        let mut lines = 0;
        let mut strbuf = "".to_string();

        // seek in reverse until seek_pos is either 0, or all required lines have been read
        while seek_pos > 0 && lines <= self.line_limit {
            let chunk_size;
            if seek_pos < self.chunk_size {
                chunk_size = seek_pos;
                seek_pos = 0;
            } else {
                seek_pos = seek_pos - self.chunk_size;
                chunk_size = self.chunk_size;
            }

            self.input.seek(SeekFrom::Start(seek_pos))?;
            let (line_count, mut result) = self.read_chunk(chunk_size as usize)?;

            result.push_str(&strbuf);
            strbuf = result;
            lines += line_count;
        }

        let mut slice = &strbuf[..];
        // count new lines and trim
        while self.count_lines(slice) >= self.line_limit {
            match self.trim(slice) {
                Some(pos) => slice = &slice[pos + 1..],
                _ => break, // out of lines
            }
        }

        return Ok(slice.into());
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileDataSource {
    line_limit: usize,
    path: String,
}

impl FileDataSource {
    pub fn new(path: &str, line_limit: usize) -> Self {
        Self {
            path: path.into(),
            line_limit,
        }
    }
}

#[typetag::serde]
#[async_trait]
impl DataSource for FileDataSource {
    async fn load(&mut self) -> BoxResult<String> {
        let file = File::open(Path::new(&self.path))?;
        let mut rev_reader = TailReader::new(file, self.line_limit);

        return Ok(rev_reader.read_lines()?);
    }
}

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
    async fn load(&mut self) -> BoxResult<String> {
        Ok(reqwest::get(&self.url).await?.text().await?)
    }
}

// TODO this be tested in a sane way at all?

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn it_should_read_files_in_reverse_up_to_limit() {
        let mut rev_reader =
            TailReader::new(Cursor::new("Data\nWith\nNew\nLines\nFor\nUnit\nTests"), 3);
        rev_reader.chunk_size = 4; // for testing we lower chunk size

        let lines = rev_reader.read_lines().unwrap();
        assert_eq!(lines, "For\nUnit\nTests");
    }

    #[test]
    fn it_should_read_files_in_reverse_up_to_start() {
        let mut rev_reader = TailReader::new(Cursor::new("Data\nWith\nNew\nLines"), 10);
        rev_reader.chunk_size = 4; // for testing we lower chunk size

        let lines = rev_reader.read_lines().unwrap();
        assert_eq!(lines, "Data\nWith\nNew\nLines");
    }

    #[test]
    fn it_should_read_files_in_reverse_up_to_start_when_chunk_is_larger_than_file() {
        let mut rev_reader = TailReader::new(Cursor::new("Data\nWith\nNew\nLines"), 10);

        let lines = rev_reader.read_lines().unwrap();
        assert_eq!(lines, "Data\nWith\nNew\nLines");
    }
}
