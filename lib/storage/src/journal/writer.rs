use bytes::buf::Writer;
use bytes::BytesMut;
use tokio::io::AsyncWrite;

use super::{Entry, JournalWriter, Result};

pub struct JournalWriterImpl<W> {
    writer: W,
    size: u64,
}

#[async_trait::async_trait]
impl<W: AsyncWrite + Send> JournalWriter for JournalWriterImpl<W> {
    fn size(&self) -> u64 {
        self.size
    }

    async fn append_entry<E: Entry + Send>(&mut self, entry: E) -> Result<()> {
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
