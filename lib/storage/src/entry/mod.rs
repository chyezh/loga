mod error;
mod header;
mod impls_v1;
mod util;

use bytes::{Buf, BufMut};
pub use error::Error;
pub use header::Header;
pub use util::{Attr, Magic};

use self::impls_v1::{BuilderV1, EntryV1};
pub type Result<T> = std::result::Result<T, Error>;

/// decode an entry from a buffer.
pub fn decode<B: Buf>(mut buf: B) -> Result<impl Entry> {
    let magic = Magic::try_from(buf.get_u8())?;
    match magic {
        Magic::V1 => EntryV1::decode_without_magic(magic, buf),
    }
}

pub trait Entry {
    /// Returns the magic of the entry.
    fn magic(&self) -> Magic;

    /// Returns the attribute of the entry.
    fn attr(&self) -> Attr;

    /// Returns the log ID of the entry.
    fn log_id(&self) -> i64;

    /// Returns the entry ID of the entry.
    fn entry_id(&self) -> i64;

    /// Returns the last confirm id of the entry.
    fn last_confirm_id(&self) -> i64;

    /// Returns the key of the entry.
    fn key(&self) -> &bytes::Bytes;

    /// Returns the value of the entry.
    fn value(&self) -> &bytes::Bytes;

    /// Returns the headers of the entry.
    fn headers(&self) -> &[Header];

    /// Get the binary size of the entry.
    fn binary_size(&self) -> usize;

    /// Encodes the entry into a buffer.
    fn encode<B: BufMut>(&self, buf: B) -> Result<()>;

    /// Decodes the buffer into an entry.
    fn decode_without_magic<B: Buf>(magic: Magic, buf: B) -> Result<Self>
    where
        Self: Sized;

    /// Read at a specific offset of Entry's binary representation.
    fn read_at(&self, buf: &mut [u8], offset: usize) -> usize;
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

        let entry = BuilderV1::new()
            .log_id(1)
            .entry_id(2)
            .attr(Attr::default())
            .last_confirm_id(3)
            .kv(key.clone(), value.clone())
            .header(header.clone())
            .build();

        assert_eq!(entry.magic(), Magic::V1);
        assert_eq!(entry.attr(), Attr::default());
        assert_eq!(entry.log_id(), 1);
        assert_eq!(entry.entry_id(), 2);
        assert_eq!(entry.last_confirm_id(), 3);
        assert_eq!(entry.key(), &key);
        assert_eq!(entry.value(), &value);
        assert_eq!(entry.headers().len(), 1);
        assert_eq!(entry.headers()[0].key(), header.key());
        assert_eq!(entry.headers()[0].value(), header.value());
    }

    #[test]
    #[should_panic(expected = "missing kv field in entry")]
    fn test_entry_builder_build_unwrap_none() {
        let builder = BuilderV1::new();
        builder.build(); // This should panic because we didn't set any values
    }

    #[test]
    fn test_entry_encode_decode() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let builder = BuilderV1::new()
            .log_id(1)
            .entry_id(2)
            .attr(Attr::default())
            .last_confirm_id(3)
            .kv(key.clone(), value.clone())
            .header(header.clone());

        let entry = builder.build();

        // Encode the entry into a buffer
        let mut buf = BytesMut::new();
        entry.encode(&mut buf).unwrap();

        // Decode the buffer back into an entry
        let decoded_entry = decode(buf.freeze()).unwrap();

        // Check that the decoded entry is the same as the original entry
        assert_eq!(decoded_entry.magic(), entry.magic());
        assert_eq!(decoded_entry.log_id(), entry.log_id());
        assert_eq!(decoded_entry.entry_id(), entry.entry_id());
        assert_eq!(decoded_entry.attr(), entry.attr());
        assert_eq!(decoded_entry.last_confirm_id(), entry.last_confirm_id());
        assert_eq!(decoded_entry.key(), entry.key());
        assert_eq!(decoded_entry.value(), entry.value());
        assert_eq!(decoded_entry.headers().len(), entry.headers().len());
        for i in 0..decoded_entry.headers().len() {
            assert_eq!(decoded_entry.headers()[i].key(), entry.headers()[i].key());
            assert_eq!(
                decoded_entry.headers()[i].value(),
                entry.headers()[i].value()
            );
        }
    }

    #[test]
    fn test_binary_size() {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let builder = BuilderV1::new()
            .log_id(1)
            .entry_id(2)
            .attr(Attr::default())
            .last_confirm_id(3)
            .kv(key.clone(), value.clone())
            .header(header.clone());

        let entry = builder.build();

        // Calculate the binary size
        let binary_size = entry.binary_size();

        // Check that the binary size is correct
        // Magic::V1 size + log_id size + entry_id size + attr size + last_confirm size + kv size + headers size
        let expected_size = 1
            + 4
            + 8
            + 8
            + 8
            + (key.len() + value.len() + 2)
            + (header.key().len() + header.value().len() + 2);
        assert_eq!(binary_size, expected_size);
    }

    #[test]
    fn test_read_at() {
        for i in 1..16 {
            read_all(i);
        }
    }

    fn read_all(step: usize) {
        let key = Bytes::from_static(b"key");
        let value = Bytes::from_static(b"value");
        let header = Header::new(key.clone(), value.clone());

        let entry = BuilderV1::new()
            .log_id(1)
            .entry_id(2)
            .attr(Attr::default())
            .last_confirm_id(3)
            .kv(key.clone(), value.clone())
            .header(header.clone())
            .header(header)
            .build();

        let mut buf = vec![0; step];
        let mut finial_decoded = Vec::new();
        let mut n = 0;
        loop {
            let k = entry.read_at(&mut buf[..], n);
            if k == 0 {
                break;
            }
            n += k;
            finial_decoded.extend_from_slice(&buf[..k]);
        }
        assert_eq!(finial_decoded.len(), entry.binary_size());

        // Encode the entry into a buffer
        let mut buf = BytesMut::new();
        entry.encode(&mut buf).unwrap();
        let buf = buf.freeze();

        assert_eq!(buf.len(), finial_decoded.len());

        // Decode the buffer back into an entry
        let decoded_entry = decode(buf).unwrap();

        // Check that the decoded entry is the same as the original entry
        assert_eq!(decoded_entry.magic(), entry.magic());
        assert_eq!(decoded_entry.log_id(), entry.log_id());
        assert_eq!(decoded_entry.entry_id(), entry.entry_id());
        assert_eq!(decoded_entry.attr(), entry.attr());
        assert_eq!(decoded_entry.last_confirm_id(), entry.last_confirm_id());
        assert_eq!(decoded_entry.key(), entry.key());
        assert_eq!(decoded_entry.value(), entry.value());
        assert_eq!(decoded_entry.headers().len(), entry.headers().len());
        for i in 0..decoded_entry.headers().len() {
            assert_eq!(decoded_entry.headers()[i].key(), entry.headers()[i].key());
            assert_eq!(
                decoded_entry.headers()[i].value(),
                entry.headers()[i].value()
            );
        }
    }
}
