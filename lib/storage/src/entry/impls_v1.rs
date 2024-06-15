use crate::util::copy_slice;

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
/// * `headers` - A vector of `Header` instances that represents the headers of the entry.
/// * `key` - A `Bytes` instance that represents the keys of the entry.
/// * `value` - A `Bytes` instance that represents the values of the entry.
pub struct EntryV1 {
    pub attr: Attr,
    pub log_id: i64,
    pub entry_id: i64,
    pub last_confirm: i64,
    pub headers: Vec<Header>,
    pub kv: Header,
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

    fn binary_size(&self) -> usize {
        let mut size = Self::common_header_binary_size();
        for header in &self.headers {
            let header_size = header.binary_size();
            size += prost::length_delimiter_len(header_size);
            size += header_size;
        }
        let kv_size = self.kv.binary_size();
        size += prost::length_delimiter_len(kv_size);
        size += kv_size;
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
        for header in &self.headers {
            let size = header.binary_size();
            prost::encode_length_delimiter(size, &mut buf)?;
            header.encode(&mut buf)?;
        }
        let size = self.kv.binary_size();
        prost::encode_length_delimiter(size, &mut buf)?;
        self.kv.encode(&mut buf)?;
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
        // Read the value from the buffer
        let mut headers = Vec::new();
        while buf.has_remaining() {
            // Decode the length of the value from the buffer
            let length = prost::decode_length_delimiter(&mut buf)?;
            let mut header_buf = buf.take(length);
            headers.push(Header::decode(&mut header_buf)?);
            buf = header_buf.into_inner();
        }
        let kv: Header = headers.pop().expect("missing kv field in entry");
        Ok(Self {
            attr,
            log_id,
            entry_id,
            last_confirm,
            kv,
            headers,
        })
    }

    fn read_at(&self, buf: &mut [u8], offset: usize) -> usize {
        let mut n = 0;
        if offset < Self::common_header_binary_size() {
            n += self.read_common_header_at_offset(buf, offset);
            if n == buf.len() {
                return n;
            }
        }
        let mut offset = offset - Self::common_header_binary_size();
        for header in &self.headers {
            (offset, n) = Self::read_at_header(header, offset, buf, n);
            if n == buf.len() {
                return n;
            }
        }
        (_, n) = Self::read_at_header(&self.kv, offset, buf, n);
        n
    }
}

impl EntryV1 {
    fn read_at_header(
        header: &Header,
        mut offset: usize,
        buf: &mut [u8],
        mut n: usize,
    ) -> (usize, usize) {
        let header_size = header.binary_size();
        let size_of_header_size_delimiter = prost::length_delimiter_len(header_size);
        if offset < size_of_header_size_delimiter {
            let mut tmp_storage = Vec::with_capacity(header_size);
            prost::encode_length_delimiter(header_size, &mut tmp_storage).unwrap();
            n += copy_slice(&tmp_storage[offset..], &mut buf[n..]);
            if n == buf.len() {
                return (offset, n);
            }
        }
        offset -= size_of_header_size_delimiter;

        if offset < header_size {
            n += header.read_at(buf, offset);
            if n == buf.len() {
                return (offset, n);
            }
        }
        offset -= header_size;
        (offset, n)
    }

    /// Get the size of the common header.
    fn common_header_binary_size() -> usize {
        // 1 +
        // 4 +
        // 8 +
        // 8 +
        // 8 = 29
        29
    }

    fn read_common_header_at_offset(&self, buf: &mut [u8], offset: usize) -> usize {
        let mut n = 0;
        if offset < 1 && !buf.is_empty() {
            buf[0] = Magic::V1.into();
            n += 1;
            if n == buf.len() {
                return n;
            }
        }
        if offset < 5 {
            let attr: i32 = self.attr.into();
            let src = attr.to_le_bytes();
            n += copy_slice(&src[offset - 1..], &mut buf[n..]);
            if n == buf.len() {
                return n;
            }
        }
        if offset < 13 {
            let src = self.log_id.to_le_bytes();
            n += copy_slice(&src[offset - 5..], &mut buf[n..]);
            if n == buf.len() {
                return n;
            }
        }
        if offset < 21 {
            let src = self.entry_id.to_le_bytes();
            n += copy_slice(&src[offset - 13..], &mut buf[n..]);
            if n == buf.len() {
                return n;
            }
        }
        let src = self.last_confirm.to_le_bytes();
        n += copy_slice(&src[offset - 21..], &mut buf[n..]);
        n
    }
}
