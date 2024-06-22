use crate::util::{copy_slice, copy_slice_with_multi_stage, customize_copy_slice_with_multi_stage};

use super::{Entry, JournalWriter, Result};
use crc::{Crc, Digest, Table, CRC_32_ISCSI};
use std::io::Write;

static CRC_INSTANCE: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

/// JournalEntryContext is a context for writing a journal entry. consists with following fields:
/// 1. size of entry
/// 2. entry
/// 3. checksum
struct JournalEntryContext<E: Entry> {
    entry: E,
    digest: Option<Digest<'static, u32, Table<1>>>,
    checksum: [u8; 4],
}

impl<E: Entry> JournalEntryContext<E> {
    fn new(entry: E) -> Self {
        Self {
            entry,
            digest: Some(CRC_INSTANCE.digest()),
            checksum: [0; 4],
        }
    }

    /// read_at reads the write context into binary
    fn read_at(&mut self, buf: &mut [u8], mut offset: usize) -> usize {
        let mut n = 0;
        let sz = self.entry.binary_size();
        let sz_size = prost::length_delimiter_len(sz); // extra 4 bytes for crc.

        let sz_size_getter = || -> Vec<u8> {
            let mut tmp_storage = Vec::with_capacity(sz_size);
            prost::encode_length_delimiter(sz, &mut tmp_storage).unwrap();
            tmp_storage
        };

        customize_copy_slice_with_multi_stage!(
            copy_slice(&sz_size_getter(), &mut buf[n..]),
            sz_size,
            buf,
            offset,
            n
        );

        customize_copy_slice_with_multi_stage!(
            self.entry.read_at(&mut buf[n..], offset),
            sz,
            buf,
            offset,
            n
        );

        // write checksum
        n += copy_slice(&self.get_checksum()[offset..], &mut buf[n..]);
        n
    }

    fn get_checksum(&mut self) -> &[u8] {
        if let Some(digest) = self.digest.take() {
            copy_slice(&digest.finalize().to_le_bytes(), &mut self.checksum);
        }
        self.checksum.as_ref()
    }
}

pub struct JournalWriterImpl {
    file: std::fs::File,
    offset: usize,
    buffer: Vec<u8>,
    size: usize,
}

impl JournalWriter for JournalWriterImpl {
    fn size(&self) -> usize {
        self.size
    }

    fn append_entry<E: Entry>(&mut self, entry: E) -> Result<()> {
        let mut entry_context = JournalEntryContext::new(entry);
        let mut offset = 0;
        loop {
            // flush the buffer if it's full before append entry into buffer.
            if self.buffer.len() == self.offset {
                self.flush()?
            }
            let mut done = false;
            (offset, done) = self.append_entry_into_buffer(&mut entry_context, offset);

            // if done, break the loop.
            if done {
                return Ok(());
            }
        }
    }

    fn sync(&mut self) -> Result<()> {
        self.file.sync_data()?;
        Ok(())
    }
}

impl JournalWriterImpl {
    /// append_entry_into_buffer appends an entry into the buffer at the given offset.
    fn append_entry_into_buffer<E: Entry>(
        &mut self,
        entry_context: &mut JournalEntryContext<E>,
        offset: usize,
    ) -> (usize, bool) {
        let k = entry_context.read_at(&mut self.buffer[self.offset..], offset);
        self.offset += k;
        self.size += k;
        (k + offset, k == 0)
    }

    /// flush writes the buffer to the underlying writer and do a flush operation.
    fn flush(&mut self) -> Result<()> {
        self.file.write_all(&self.buffer[..self.offset])?;
        self.offset = 0;
        Ok(())
    }
}
