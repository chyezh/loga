mod error;
mod writer;

use crate::entry::Entry;
use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

// WriteInfo is a struct that contains the sequence number and offset of a write operation.
pub struct WriteInfo {
    seq: u64,
    offset: u64,
}

/// Trait representing a writer of journal.
/// Journal is a sequence of entries, where each entry is a record of some event.
pub trait JournalWriter {
    /// the size of current journal in bytes.
    fn size(&self) -> usize;

    /// Appends an entry to the journal.
    ///
    /// # Arguments
    ///
    /// * `entry` - The entry to be appended.
    fn append_entry<E: Entry + Send>(&mut self, entry: E) -> Result<()>;

    /// sync the journal, ensuring all entries are written to the underlying reliable storage.
    fn sync(&mut self) -> Result<()>;
}

/// Trait representing a reader of journal.
pub trait JournalReader {
    fn next(&self) -> Result<impl Entry>;
}
