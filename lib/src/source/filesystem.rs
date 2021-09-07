use crate::async_trait::async_trait;
use crate::error::Error;
use crate::serde::{Deserialize, Serialize};
use crate::typetag;
use crate::DataSource;
use std::path::Path;
use std::str;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, SeekFrom};

/**
 * File data input
 */

/**
 * Takes an input buffer and reads the bottom n lines backwards
 */
pub struct TailReader<T>
where
    T: AsyncRead + AsyncSeek + std::marker::Unpin,
{
    input: T,
    line_limit: usize,
    chunk_size: u64,
}

impl<T> TailReader<T>
where
    T: AsyncRead + AsyncSeek + std::marker::Unpin,
{
    pub fn new(input: T, line_limit: usize) -> Self {
        Self {
            input,
            line_limit,
            chunk_size: 64,
        }
    }

    async fn read_chunk(&mut self, chunk_size: usize) -> Result<(usize, String), Error> {
        let mut buf = vec![0u8; chunk_size];
        self.input.read_exact(&mut buf).await?;

        let result = str::from_utf8(&buf)?;

        Ok((self.count_lines(&result), result.into()))
    }

    fn count_lines(&self, strbuf: &str) -> usize {
        strbuf.matches("\n").count()
    }

    fn trim(&self, strbuf: &str) -> Option<usize> {
        strbuf.find("\n")
    }

    pub async fn read_lines(&mut self) -> Result<String, Error> {
        // seek backwards in chunk increments
        // until enough new line characters have been found

        // get total file size
        let mut seek_pos = self.input.seek(SeekFrom::End(0)).await?;

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

            self.input.seek(SeekFrom::Start(seek_pos)).await?;
            let (line_count, mut result) = self.read_chunk(chunk_size as usize).await?;

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
    async fn load(&mut self) -> Result<String, Error> {
        let file = File::open(Path::new(&self.path)).await?;
        let mut rev_reader = TailReader::new(file, self.line_limit);

        return Ok(rev_reader.read_lines().await?);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn it_should_read_files_in_reverse_up_to_limit() {
        let mut rev_reader =
            TailReader::new(Cursor::new("Data\nWith\nNew\nLines\nFor\nUnit\nTests"), 3);
        rev_reader.chunk_size = 4; // for testing we lower chunk size

        let lines = rev_reader.read_lines().await.unwrap();
        assert_eq!(lines, "For\nUnit\nTests");
    }

    #[tokio::test]
    async fn it_should_read_files_in_reverse_up_to_start() {
        let mut rev_reader = TailReader::new(Cursor::new("Data\nWith\nNew\nLines"), 10);
        rev_reader.chunk_size = 4; // for testing we lower chunk size

        let lines = rev_reader.read_lines().await.unwrap();
        assert_eq!(lines, "Data\nWith\nNew\nLines");
    }

    #[tokio::test]
    async fn it_should_read_files_in_reverse_up_to_start_when_chunk_is_larger_than_file() {
        let mut rev_reader = TailReader::new(Cursor::new("Data\nWith\nNew\nLines"), 10);

        let lines = rev_reader.read_lines().await.unwrap();
        assert_eq!(lines, "Data\nWith\nNew\nLines");
    }
}
