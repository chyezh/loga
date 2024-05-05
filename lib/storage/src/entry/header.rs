// Importing necessary modules and types
use super::Result;
use bytes::{Buf, BufMut, Bytes};

// Defining a struct Header with key and value as Bytes
// It use length delimited encoding
#[derive(Clone)]
pub struct Header {
    key: Bytes,
    value: Bytes,
}

// Implementing methods for Header struct
impl Header {
    // Constructor for Header
    pub fn new(key: Bytes, value: Bytes) -> Self {
        Self { key, value }
    }

    // Getter for key
    pub fn key(&self) -> &Bytes {
        &self.key
    }

    // Getter for value
    pub fn value(&self) -> &Bytes {
        &self.value
    }

    // Method to estimate the size of the Header
    pub fn estimate_size(&self) -> usize {
        let key_len = self.key.len();
        let value_len = self.value.len();
        prost::length_delimiter_len(key_len)
            + prost::length_delimiter_len(value_len)
            + self.key.len()
            + self.value.len()
    }

    // Method to encode the Header into a buffer
    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        // Get the length of the key
        let length = self.key.len();
        // Write the length of the key to the buffer
        prost::encode_length_delimiter(length, buf)?;
        // Write the key to the buffer
        buf.put_slice(&self.key[..]);
        // Get the length of the value
        let length = self.value.len();
        // Write the length of the value to the buffer
        prost::encode_length_delimiter(length, buf)?;
        // Write the value to the buffer
        buf.put_slice(&self.value[..]);
        Ok(())
    }

    // Method to decode the Header from a buffer
    pub fn decode<B: Buf>(mut buf: B) -> Result<Self> {
        // Decode the length of the key from the buffer
        let length = prost::decode_length_delimiter(&mut buf)?;
        // Read the key from the buffer
        let key = buf.copy_to_bytes(length);
        // Decode the length of the value from the buffer
        let length = prost::decode_length_delimiter(&mut buf)?;
        // Read the value from the buffer
        let value = buf.copy_to_bytes(length);
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
    fn test_estimate_size() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        assert_eq!(header.estimate_size(), 2 + key.len() + value.len());
    }

    #[test]
    fn test_header_encode() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let mut buf = BytesMut::new();
        header.encode(&mut buf).unwrap();

        // Check the encoded length and value
        assert_eq!(buf.len(), key.len() + value.len() + 2); // 4 for two length delimiters
        assert_eq!(&buf[1..4], &key[..]);
        assert_eq!(&buf[5..], &value[..]);
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
}
