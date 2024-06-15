mod error;
mod writer;

use crate::entry::Entry;
use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

/// Trait representing a writer of journal.
/// Journal is a sequence of entries, where each entry is a record of some event.
#[async_trait::async_trait]
pub trait JournalWriter {
    /// the size of current journal in bytes.
    fn size(&self) -> u64;

    /// Appends an entry to the journal.
    ///
    /// # Arguments
    ///
    /// * `entry` - The entry to be appended.
    async fn append_entry<E: Entry + Send>(&mut self, entry: E) -> Result<()>;

    /// Flushes the journal, ensuring all entries are written to the underlying reliable storage.
    async fn flush(&mut self) -> Result<()>;
}

/// Trait representing a reader of journal.
#[async_trait::async_trait]
pub trait JournalReader {
    async fn next(&self) -> Result<impl Entry>;
}
