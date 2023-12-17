mod error;
mod header;
mod util;

pub use error::Error;
pub use header::Header;
pub use util::Attr;
pub type Result<T> = std::result::Result<T, Error>;

use bytes::{Buf, BufMut, Bytes};
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
    kv: Header,
    headers: Vec<Header>,
}

impl Entry {
    // Getter for attr
    pub fn attr(&self) -> Attr {
        self.attr
    }

    // Getter for log_id
    pub fn log_id(&self) -> i64 {
        self.log_id
    }

    // Getter for entry_id
    pub fn entry_id(&self) -> i64 {
        self.entry_id
    }

    // Getter for last_confirm
    pub fn last_confirm(&self) -> i64 {
        self.last_confirm
    }

    // Getter for key
    pub fn key(&self) -> &Bytes {
        &self.kv.key
    }

    // Getter for value
    pub fn value(&self) -> &Bytes {
        &self.kv.value
    }

    // Getter for headers
    pub fn headers(&self) -> &[Header] {
        &self.headers
    }

    // Method to estimate the size of the Entry
    pub fn estimate_size(&self) -> usize {
        let mut size = 0;
        size += 1; // magic
        size += 4; // attr
        size += 8; // log_id
        size += 8; // entry_id
        size += 8; // last_confirm
        size += self.kv.estimate_size();
        for header in &self.headers {
            size += header.estimate_size();
        }
        size
    }

    // Method to encode the Entry into a buffer
    pub fn encode<B: BufMut>(&self, mut buf: B) -> Result<()> {
        // Write the magic to the buffer
        buf.put_u8(self.magic.into());
        // Write the attr to the buffer
        buf.put_i32(self.attr.into());
        // Write the log id to the buffer
        buf.put_i64(self.log_id);
        // Write the entry id to the buffer
        buf.put_i64(self.entry_id);
        // Write the last confirm to the buffer
        buf.put_i64(self.last_confirm);
        let mut size = self.kv.estimate_size();
        for header in &self.headers {
            size += header.estimate_size();
        }
        prost::encode_length_delimiter(size, &mut buf)?;
        self.kv.encode(&mut buf)?;
        for header in &self.headers {
            header.encode(&mut buf)?;
        }
        Ok(())
    }

    pub fn decode<B: Buf>(mut buf: B) -> Result<Self> {
        // TODO: check exists.

        // Read the magic from the buffer
        let magic = Magic::from(buf.get_u8());
        // Read the attr from the buffer
        let attr = Attr::from(buf.get_i32());
        // Read the log id from the buffer
        let log_id = buf.get_i64();
        // Read the entry id from the buffer
        let entry_id = buf.get_i64();
        // Read the last confirm from the buffer
        let last_confirm = buf.get_i64();
        // Decode the length of the value from the buffer
        let length = prost::decode_length_delimiter(&mut buf)?;
        let mut buf = buf.take(length);
        // Read the value from the buffer
        let kv = Header::decode(&mut buf)?;
        let mut headers = Vec::new();
        while buf.has_remaining() {
            headers.push(Header::decode(&mut buf)?);
        }
        Ok(Self {
            magic,
            attr,
            log_id,
            entry_id,
            last_confirm,
            kv,
            headers,
        })
    }
}

/// The `EntryBuilder` struct provides a way to construct a new `Entry`.
#[derive(Default)]
pub struct EntryBuilder {
    log_id: Option<i64>,
    entry_id: Option<i64>,
    attr: Attr,
    last_confirm: Option<i64>,
    kv: Option<Header>,
    headers: Vec<Header>,
}

impl EntryBuilder {
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
    pub fn build(self) -> Entry {
        Entry {
            magic: Magic::V1,
            log_id: self.log_id.unwrap(),
            entry_id: self.entry_id.unwrap(),
            attr: self.attr,
            last_confirm: self.last_confirm.unwrap(),
            kv: self.kv.unwrap(),
            headers: self.headers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};

    #[test]
    fn test_entry_builder_build() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let builder = EntryBuilder::default()
            .log_id(1)
            .entry_id(2)
            .attr(Attr::default())
            .last_confirm(3)
            .kv(key.clone(), value.clone())
            .header(header.clone());

        let entry = builder.build();

        assert_eq!(entry.magic, Magic::V1);
        assert_eq!(entry.log_id, 1);
        assert_eq!(entry.entry_id, 2);
        assert_eq!(entry.attr, Attr::default());
        assert_eq!(entry.last_confirm, 3);
        assert_eq!(entry.kv.key(), &key);
        assert_eq!(entry.kv.value(), &value);
        assert_eq!(entry.headers.len(), 1);
        assert_eq!(entry.headers[0].key(), header.key());
        assert_eq!(entry.headers[0].value(), header.value());
        assert_eq!(entry.attr(), Attr::default());
        assert_eq!(entry.log_id(), 1);
        assert_eq!(entry.entry_id(), 2);
        assert_eq!(entry.last_confirm(), 3);
        assert_eq!(entry.key(), &key);
        assert_eq!(entry.value(), &value);
        assert_eq!(entry.headers().len(), 1);
        assert_eq!(entry.headers()[0].key(), header.key());
        assert_eq!(entry.headers()[0].value(), header.value());
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn test_entry_builder_build_unwrap_none() {
        let builder = EntryBuilder::default();
        builder.build(); // This should panic because we didn't set any values
    }

    #[test]
    fn test_entry_encode_decode() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let builder = EntryBuilder::default()
            .log_id(1)
            .entry_id(2)
            .attr(Attr::default())
            .last_confirm(3)
            .kv(key.clone(), value.clone())
            .header(header.clone());

        let entry = builder.build();

        // Encode the entry into a buffer
        let mut buf = BytesMut::new();
        entry.encode(&mut buf).unwrap();

        // Decode the buffer back into an entry
        let decoded_entry = Entry::decode(buf.freeze()).unwrap();

        // Check that the decoded entry is the same as the original entry
        assert_eq!(decoded_entry.magic, entry.magic);
        assert_eq!(decoded_entry.log_id, entry.log_id);
        assert_eq!(decoded_entry.entry_id, entry.entry_id);
        assert_eq!(decoded_entry.attr, entry.attr);
        assert_eq!(decoded_entry.last_confirm, entry.last_confirm);
        assert_eq!(decoded_entry.kv.key(), entry.kv.key());
        assert_eq!(decoded_entry.kv.value(), entry.kv.value());
        assert_eq!(decoded_entry.headers.len(), entry.headers.len());
        for i in 0..decoded_entry.headers.len() {
            assert_eq!(decoded_entry.headers[i].key(), entry.headers[i].key());
            assert_eq!(decoded_entry.headers[i].value(), entry.headers[i].value());
        }
    }

    #[test]
    fn test_estimate_size() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let builder = EntryBuilder::default()
            .log_id(1)
            .entry_id(2)
            .attr(Attr::default())
            .last_confirm(3)
            .kv(key.clone(), value.clone())
            .header(header.clone());

        let entry = builder.build();

        // Calculate the estimated size
        let estimated_size = entry.estimate_size();

        // Check that the estimated size is correct
        // Magic::V1 size + log_id size + entry_id size + attr size + last_confirm size + kv size + headers size
        let expected_size = 1
            + 4
            + 8
            + 8
            + 8
            + (key.len() + value.len() + 2)
            + (header.key().len() + header.value().len() + 2);
        assert_eq!(estimated_size, expected_size);
    }
}
