use bytes::{Buf, BufMut, Bytes};

use crate::util::copy_slice;

use super::Result;
use super::{util::copy_slice_with_multi_stage, util::customize_copy_slice_with_multi_stage};

// Defining a struct Header with key and value as Bytes
// It use length delimited encoding
#[derive(Clone)]
pub struct Header {
    key: Bytes,
    value: Bytes,
}

// Implementing methods for Header struct
impl Header {
    /// Constructor for Header
    pub fn new(key: Bytes, value: Bytes) -> Self {
        Self { key, value }
    }

    /// Getter for key
    pub fn key(&self) -> &Bytes {
        &self.key
    }

    /// Getter for value
    pub fn value(&self) -> &Bytes {
        &self.value
    }

    /// read at a specific offset of Header's binary representation.
    pub fn read_at(&self, buf: &mut [u8], mut offset: usize) -> usize {
        let key_len = self.key.len();
        let key_len_size = prost::length_delimiter_len(key_len);
        let mut n = 0;
        let key_len_delimiter_getter = || {
            let mut tmp_storage = Vec::with_capacity(key_len_size);
            // There's enough capacity, so should never fail.
            prost::encode_length_delimiter(key_len, &mut tmp_storage).unwrap();
            tmp_storage
        };

        customize_copy_slice_with_multi_stage!(
            copy_slice(&key_len_delimiter_getter(), &mut buf[n..]),
            key_len_size,
            buf,
            offset,
            n
        );
        copy_slice_with_multi_stage!(self.key, buf, offset, n);
        copy_slice_with_multi_stage!(self.value, buf, offset, n);
        n
    }

    /// Method to get the binary size of the Header
    pub fn binary_size(&self) -> usize {
        let key_len = self.key.len();
        prost::length_delimiter_len(key_len) + self.key.len() + self.value.len()
    }

    /// Method to encode the Header into a buffer
    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        // Get the length of the key
        let length = self.key.len();
        // Write the length of the key to the buffer
        prost::encode_length_delimiter(length, buf)?;
        // Write the key to the buffer
        buf.put_slice(&self.key[..]);
        // Write the value to the buffer
        buf.put_slice(&self.value[..]);
        Ok(())
    }

    /// Method to decode the Header from a buffer
    pub fn decode<B: Buf>(mut buf: B) -> Result<Self> {
        // Decode the length of the key from the buffer
        let key_len = prost::decode_length_delimiter(&mut buf)?;
        // Read the key from the buffer
        let key = buf.copy_to_bytes(key_len);
        // Read the value from the buffer
        let value = buf.copy_to_bytes(buf.remaining());
        // Return the Header
        Ok(Self { key, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use bytes::BytesMut;

    #[test]
    fn test_header_new() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        assert_eq!(header.key(), &key);
        assert_eq!(header.value(), &value);
    }

    #[test]
    fn test_binary_size() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        assert_eq!(header.binary_size(), 1 + key.len() + value.len());
    }

    #[test]
    fn test_header_encode() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let mut buf = BytesMut::new();
        header.encode(&mut buf).unwrap();

        // Check the encoded length and value
        assert_eq!(buf.len(), key.len() + value.len() + 1);
        assert_eq!(&buf[1..4], &key[..]);
        assert_eq!(&buf[4..], &value[..]);
    }

    #[test]
    fn test_header_decode() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let mut buf = BytesMut::new();
        header.encode(&mut buf).unwrap();

        let decoded_header = Header::decode(buf.freeze()).unwrap();

        assert_eq!(decoded_header.key(), &key);
        assert_eq!(decoded_header.value(), &value);
    }

    #[test]
    fn test_read_at() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let mut buf = vec![0; key.len()];
        let n = header.read_at(&mut buf, 1);
        assert_eq!(n, key.len());
        assert_eq!(&buf, &key);

        let mut buf = vec![0; value.len()];
        let n = header.read_at(&mut buf, key.len() + 1);
        assert_eq!(n, value.len());
        assert_eq!(&buf, &value);

        let mut buf = vec![0; key.len() + value.len()];
        let n = header.read_at(&mut buf, 1);
        assert_eq!(n, key.len() + value.len());
        assert_eq!(&buf[..], b"keyvalue");

        let mut buf = vec![0; key.len() + value.len() + 1];
        let n = header.read_at(&mut buf, 0);
        assert_eq!(n, header.binary_size());
        assert_eq!(buf, b"\x03keyvalue");
    }

    #[test]
    fn test_read_at_all() {
        for i in 1..10 {
            read_all(i);
        }
    }

    fn read_all(step: usize) {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let mut buf = vec![0; step];
        let mut all = vec![];
        let mut offset = 0;
        loop {
            let n = header.read_at(&mut buf, offset);
            offset += n;
            all.extend_from_slice(&buf[..n]);
            if n == 0 {
                break;
            }
        }
        assert_eq!(all, b"\x03keyvalue");
        assert_eq!(offset, header.binary_size());
    }
}
