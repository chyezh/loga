use bytes::{Buf, BufMut, Bytes};

use crate::util::copy_slice;

use super::Attr;
use super::Entry;
use super::Header;
use super::Magic;
use super::Result;
use super::{util::copy_slice_with_multi_stage, util::customize_copy_slice_with_multi_stage};

// Magic 1
// Attr 4
// log_id 8
// entry_id 8
// last_confirm_id 8 = 29
const COMMON_HEADER_BINARY_SIZE: usize = 29;
const COMMON_HEADER_MAGIC_OFFSET: usize = 0;
const COMMON_HEADER_ATTR_OFFSET: usize = 1;
const COMMON_HEADER_LOG_ID_OFFSET: usize = 5;
const COMMON_HEADER_ENTRY_ID_OFFSET: usize = 13;
const COMMON_HEADER_LAC_ID_OFFSET: usize = 21;

/// The `EntryBuilder` struct provides a way to construct a new `Entry`.
pub struct BuilderV1 {
    common_header: [u8; COMMON_HEADER_BINARY_SIZE],
    kv: Option<Header>,
    headers: Vec<Header>,
}

impl BuilderV1 {
    /// Constructor for BuilderV1
    pub fn new() -> Self {
        let mut b = BuilderV1 {
            common_header: [0; COMMON_HEADER_BINARY_SIZE],
            kv: None,
            headers: Vec::new(),
        };
        b.common_header[COMMON_HEADER_MAGIC_OFFSET] = Magic::V1.into();
        b
    }

    /// Method to set the attr of the Entry
    pub fn attr(mut self, attr: Attr) -> Self {
        self.put_i32_to_common_header(COMMON_HEADER_ATTR_OFFSET, attr.into());
        self
    }

    /// Method to set the log_id of the Entry
    pub fn log_id(mut self, log_id: i64) -> Self {
        self.put_i64_to_common_header(COMMON_HEADER_LOG_ID_OFFSET, log_id);
        self
    }

    /// Method to set the entry_id of the Entry
    pub fn entry_id(mut self, entry_id: i64) -> Self {
        self.put_i64_to_common_header(COMMON_HEADER_ENTRY_ID_OFFSET, entry_id);
        self
    }

    /// Method to set the last_confirm id of the Entry
    pub fn last_confirm_id(mut self, last_confirm_id: i64) -> Self {
        self.put_i64_to_common_header(COMMON_HEADER_LAC_ID_OFFSET, last_confirm_id);
        self
    }

    /// Method to set the kv of the EntryBuilder
    pub fn kv(mut self, key: Bytes, value: Bytes) -> Self {
        self.kv = Some(Header::new(key, value));
        self
    }

    /// Method to set the header of the EntryBuilder
    pub fn header(mut self, header: Header) -> Self {
        self.headers.push(header);
        self
    }

    /// Method to build the Entry
    pub fn build(mut self) -> impl Entry {
        self.headers
            .push(self.kv.expect("missing kv field in entry"));
        EntryV1 {
            common_header: self.common_header,
            headers: self.headers,
        }
    }

    fn put_i64_to_common_header(&mut self, offset: usize, value: i64) {
        copy_slice(
            &value.to_le_bytes(),
            &mut self.common_header[offset..offset + 8],
        );
    }

    fn put_i32_to_common_header(&mut self, offset: usize, value: i32) {
        copy_slice(
            &value.to_le_bytes(),
            &mut self.common_header[offset..offset + 4],
        );
    }
}

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
    pub common_header: [u8; COMMON_HEADER_BINARY_SIZE],
    pub headers: Vec<Header>,
}

impl Entry for EntryV1 {
    fn magic(&self) -> Magic {
        Magic::try_from(self.common_header[COMMON_HEADER_MAGIC_OFFSET]).expect("invalid magic")
    }

    fn attr(&self) -> Attr {
        Attr::from(self.get_i32_from_common_header(COMMON_HEADER_ATTR_OFFSET))
    }

    fn log_id(&self) -> i64 {
        self.get_i64_from_common_header(COMMON_HEADER_LOG_ID_OFFSET)
    }

    fn entry_id(&self) -> i64 {
        self.get_i64_from_common_header(COMMON_HEADER_ENTRY_ID_OFFSET)
    }

    fn last_confirm_id(&self) -> i64 {
        self.get_i64_from_common_header(COMMON_HEADER_LAC_ID_OFFSET)
    }

    fn key(&self) -> &Bytes {
        self.headers.last().unwrap().key()
    }

    fn value(&self) -> &Bytes {
        self.headers.last().unwrap().value()
    }

    fn headers(&self) -> &[Header] {
        &self.headers[..self.headers.len() - 1]
    }

    fn binary_size(&self) -> usize {
        let mut size = COMMON_HEADER_BINARY_SIZE;
        for header in &self.headers {
            let header_size = header.binary_size();
            size += prost::length_delimiter_len(header_size);
            size += header_size;
        }
        size
    }

    fn encode<B: BufMut>(&self, mut buf: B) -> Result<()> {
        buf.put_slice(&self.common_header);
        for header in &self.headers {
            let size = header.binary_size();
            prost::encode_length_delimiter(size, &mut buf)?;
            header.encode(&mut buf)?;
        }
        Ok(())
    }

    fn decode_without_magic<B: Buf>(magic: Magic, mut buf: B) -> Result<Self> {
        let mut common_header = [0; COMMON_HEADER_BINARY_SIZE];
        common_header[0] = magic.into();
        buf.copy_to_slice(&mut common_header[1..]);

        // Read the value from the buffer
        let mut headers = Vec::new();
        while buf.has_remaining() {
            // Decode the length of the value from the buffer
            let length = prost::decode_length_delimiter(&mut buf)?;
            let mut header_buf = buf.take(length);
            headers.push(Header::decode(&mut header_buf)?);
            buf = header_buf.into_inner();
        }
        if headers.is_empty() {
            return Err(super::Error::KVNotFound);
        }
        Ok(Self {
            common_header,
            headers,
        })
    }

    fn read_at(&self, buf: &mut [u8], mut offset: usize) -> usize {
        let mut n = 0;

        copy_slice_with_multi_stage!(self.common_header, buf, offset, n);

        for header in &self.headers {
            let header_size = header.binary_size();
            let header_size_delimiter_size = prost::length_delimiter_len(header_size);
            let header_size_delimiter_getter = || -> Vec<u8> {
                let mut tmp_storage = Vec::with_capacity(header_size_delimiter_size);
                prost::encode_length_delimiter(header_size, &mut tmp_storage).unwrap();
                tmp_storage
            };
            customize_copy_slice_with_multi_stage!(
                copy_slice(&header_size_delimiter_getter(), &mut buf[n..]),
                header_size_delimiter_size,
                buf,
                offset,
                n
            );
            customize_copy_slice_with_multi_stage!(
                header.read_at(&mut buf[n..], offset),
                header_size,
                buf,
                offset,
                n
            );
        }
        n
    }
}

impl EntryV1 {
    fn get_i64_from_common_header(&self, offset: usize) -> i64 {
        let mut buf = [0; 8];
        copy_slice(&self.common_header[offset..offset + 8], &mut buf);
        i64::from_le_bytes(buf)
    }

    fn get_i32_from_common_header(&self, offset: usize) -> i32 {
        let mut buf = [0; 4];
        copy_slice(&self.common_header[offset..offset + 4], &mut buf);
        i32::from_le_bytes(buf)
    }
}
