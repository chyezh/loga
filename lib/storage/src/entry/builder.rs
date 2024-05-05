use super::impls::EntryV1;
use super::Attr;
use super::Entry;
use super::Header;
use super::Magic;
use bytes::Bytes;

/// The `EntryBuilder` struct provides a way to construct a new `Entry`.
#[derive(Default)]
pub struct Builder {
    log_id: Option<i64>,
    entry_id: Option<i64>,
    attr: Attr,
    last_confirm: Option<i64>,
    kv: Option<Header>,
    headers: Vec<Header>,
}

impl Builder {
    // Method to set the attr of the EntryBuilder
    pub fn attr(mut self, attr: Attr) -> Self {
        self.attr = attr;
        self
    }

    // Method to set the log_id of the EntryBuilder
    pub fn log_id(mut self, log_id: i64) -> Self {
        self.log_id = Some(log_id);
        self
    }

    // Method to set the entry_id of the EntryBuilder
    pub fn entry_id(mut self, entry_id: i64) -> Self {
        self.entry_id = Some(entry_id);
        self
    }

    // Method to set the last_confirm of the EntryBuilder
    pub fn last_confirm(mut self, last_confirm: i64) -> Self {
        self.last_confirm = Some(last_confirm);
        self
    }

    // Method to set the kv of the EntryBuilder
    pub fn kv(mut self, key: Bytes, value: Bytes) -> Self {
        self.kv = Some(Header::new(key, value));
        self
    }

    // Method to set the header of the EntryBuilder
    pub fn header(mut self, header: Header) -> Self {
        self.headers.push(header);
        self
    }

    // Method to build the Entry
    pub fn build(self, magic: Magic) -> impl Entry {
        match magic {
            Magic::V1 => self.build_v1(),
        }
    }

    fn build_v1(self) -> impl Entry {
        EntryV1 {
            log_id: self.log_id.unwrap(),
            entry_id: self.entry_id.unwrap(),
            attr: self.attr,
            last_confirm: self.last_confirm.unwrap(),
            kv: self.kv.unwrap(),
            headers: self.headers,
        }
    }
}
