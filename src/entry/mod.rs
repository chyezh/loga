mod error;
mod header;
mod util;

pub use error::Error;
pub use util::Attr;
pub type Result<T> = std::result::Result<T, Error>;

use bytes::Bytes;
use util::Magic;

/// The `Entry` struct represents a log entry in the system.
///
/// # Fields
/// * `magic` - An enumurate that represents the version of a entry.
/// * `attr` - An instance of the `Attr` that represents the attributes of the entry.
/// * `log_id` - An i64 that represents the log id.
/// * `entry_id` - An i64 that represents the entry id.
/// * `last_confirm` - An i64 that represents the last confirmed entry, for LAC protocol.
/// * `key` - A `Bytes` instance that represents the keys of the entry.
/// * `value` - A `Bytes` instance that represents the values of the entry.
pub struct Entry {
    magic: Magic,
    attr: Attr,
    log_id: i64,
    entry_id: i64,
    last_confirm: i64,
    key: Bytes,
    value: Bytes,
}

impl Entry {
    /// Returns a new `EntryBuilder` instance.
    fn builder() -> EntryBuilder {
        EntryBuilder::default()
    }
}

/// The `EntryBuilder` struct provides a way to construct a new `Entry`.
#[derive(Default)]
struct EntryBuilder {
    log_id: Option<i64>,
    entry_id: Option<i64>,
    attr: Option<Attr>,
    last_confirm: Option<i64>,
    key: Option<Bytes>,
    value: Option<Bytes>,
}

impl EntryBuilder {
    pub fn attr(mut self, attr: Attr) -> Self {
        self.attr = Some(attr);
        self
    }

    pub fn log_id(mut self, log_id: i64) -> Self {
        self.log_id = Some(log_id);
        self
    }

    pub fn entry_id(mut self, entry_id: i64) -> Self {
        self.entry_id = Some(entry_id);
        self
    }

    pub fn last_confirm(mut self, last_confirm: i64) -> Self {
        self.last_confirm = Some(last_confirm);
        self
    }

    pub fn key(mut self, keys: Bytes) -> Self {
        self.key = Some(keys);
        self
    }

    pub fn value(mut self, values: Bytes) -> Self {
        self.value = Some(values);
        self
    }

    pub fn build(self) -> Entry {
        Entry {
            magic: Magic::V1,
            log_id: self.log_id.unwrap(),
            entry_id: self.entry_id.unwrap(),
            attr: self.attr.unwrap(),
            last_confirm: self.last_confirm.unwrap(),
            key: self.key.unwrap(),
            value: self.value.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_builder() {
        let entry = Entry::builder()
            .log_id(1)
            .entry_id(2)
            .attr(Attr::default())
            .last_confirm(3)
            .key(Bytes::from("key"))
            .value(Bytes::from("value"))
            .build();

        assert_eq!(entry.magic, Magic::V1);
        assert_eq!(entry.log_id, 1);
        assert_eq!(entry.entry_id, 2);
        assert_eq!(entry.attr, Attr::default());
        assert_eq!(entry.last_confirm, 3);
        assert_eq!(entry.key, Bytes::from("key"));
        assert_eq!(entry.value, Bytes::from("value"));
    }
}
