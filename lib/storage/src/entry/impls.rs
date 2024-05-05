use super::Attr;
use super::Entry;
use super::Header;
use super::Magic;
use super::Result;
use bytes::{Buf, BufMut, Bytes};

/// The `Entry` struct represents a log entry in the system.
///
/// # Fields
/// * `attr` - An instance of the `Attr` that represents the attributes of the entry.
/// * `log_id` - An i64 that represents the log id.
/// * `entry_id` - An i64 that represents the entry id.
/// * `last_confirm` - An i64 that represents the last confirmed entry, for LAC protocol.
/// * `key` - A `Bytes` instance that represents the keys of the entry.
/// * `value` - A `Bytes` instance that represents the values of the entry.
pub struct EntryV1 {
    pub attr: Attr,
    pub log_id: i64,
    pub entry_id: i64,
    pub last_confirm: i64,
    pub kv: Header,
    pub headers: Vec<Header>,
}

impl Entry for EntryV1 {
    fn magic(&self) -> Magic {
        Magic::V1
    }

    fn attr(&self) -> Attr {
        self.attr
    }

    fn log_id(&self) -> i64 {
        self.log_id
    }

    fn entry_id(&self) -> i64 {
        self.entry_id
    }

    fn last_confirm(&self) -> i64 {
        self.last_confirm
    }

    fn key(&self) -> &Bytes {
        self.kv.key()
    }

    fn value(&self) -> &Bytes {
        self.kv.value()
    }

    fn headers(&self) -> &[Header] {
        &self.headers
    }

    fn estimate_size(&self) -> usize {
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

    fn encode<B: BufMut>(&self, mut buf: B) -> Result<()> {
        // Write the magic to the buffer
        buf.put_u8(Magic::V1.into());
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

    fn decode_without_magic<B: Buf>(mut buf: B) -> Result<Self> {
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
            attr,
            log_id,
            entry_id,
            last_confirm,
            kv,
            headers,
        })
    }
}
