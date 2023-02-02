use std::io::{self, Read};
use std::ptr;

/// Buffer for reading into.
///
/// This type is very similar to [`BufReader`](std::io::BufReader), but with
/// with the added ability to allow for users to request more data to be filled
/// into the buffer even when the buffer isn't empty using [`ReadBuf::read`].
#[derive(Debug, Default)]
pub struct ReadBuf {
    buf: Box<[u8]>,
    start: usize,
    end: usize,
}

impl ReadBuf {
    /// Creates a new [`ReadBuf`] with a capacity of 4096.
    pub fn new() -> Self {
        Self::with_capacity(4096)
    }

    /// Creates a new [`ReadBuf`] with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        ReadBuf {
            buf: vec![0; capacity].into_boxed_slice(),
            start: 0,
            end: 0,
        }
    }

    /// Reads some more bytes into the buffer, returning the number of bytes
    /// read.
    ///
    /// This method calls [`Read::read`] on the provided reader, but automatically
    /// provides a `&mut [u8]` for it to read into. The resulting buffer can
    /// then be accessed with [`ReadBuf::buf`].
    pub fn read<R: Read>(&mut self, mut reader: R) -> io::Result<usize> {
        if self.end + 512 > self.buf.len() {
            // Remove garbage
            // SAFETY: `self.start` and `self.end` are both valid indices into
            // `self.buf`.
            unsafe {
                ptr::copy(
                    self.buf.as_ptr().add(self.start),
                    self.buf.as_mut_ptr(),
                    self.end - self.start,
                );
            }
            self.end -= self.start;
            self.start = 0;
        }

        let len = reader.read(&mut self.buf[self.end..])?;
        self.end += len;

        if len > 0 {
            Ok(len)
        } else {
            Err(io::Error::from(io::ErrorKind::WriteZero))
        }
    }

    /// Returns the bytes currently buffered.
    pub fn buf(&self) -> &[u8] {
        &self.buf[self.start..self.end]
    }

    /// Marks `amt` bytes as consumed.
    ///
    /// # Panics
    ///
    /// This method panics if there aren't `amt` bytes in the buffer.
    pub fn consume(&mut self, amt: usize) {
        assert!(self.end - self.start >= amt, "not enough bytes to consume");
        self.start += amt;
    }
}
