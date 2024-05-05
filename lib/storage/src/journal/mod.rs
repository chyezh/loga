mod error;

use super::entry::Entry;
use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

/// Trait representing a writer of journal.
#[async_trait::async_trait]
pub trait JournalWriter {
    /// Appends an entry to the journal.
    ///
    /// # Arguments
    ///
    /// * `entry` - The entry to be appended.
    async fn append_entry<E: Entry>(&self, entry: E) -> Result<()>;

    /// Flushes the journal, ensuring all entries are written to the underlying storage.
    async fn flush(&self) -> Result<()>;
}

/// Trait representing a reader of journal.
#[async_trait::async_trait]
pub trait JournalReader {
    async fn next(&self) -> Result<impl Entry>;
}
