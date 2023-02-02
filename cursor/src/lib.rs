//! Utilities for working with [`Cursor<&[u8]>`](Cursor)s.
use std::fmt;
use std::io::Cursor;

/// Error type for reading bytes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CursorError {
    /// Expected an `\r\n`-terminated line, but no terminating `\r\n` was found
    /// with the current data in the buffer.
    Unterminated(usize),
    /// A specific number of bytes were requested (like through [`byte`] or [`slice()`]),
    /// but there weren't that many bytes remaining.
    Incomplete,
    /// `i64` not parsable from ASCII.
    Integer,
    /// `u64` not parsable from ASCII.
    Size,
}

impl CursorError {
    pub fn not_enough_data(&self) -> bool {
        matches!(self, Self::Unterminated(_) | Self::Incomplete)
    }
}

impl fmt::Display for CursorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unterminated(line_len) => {
                write!(f, "unterminated line of {line_len} bytes so far")
            }
            Self::Incomplete => "incomplete".fmt(f),
            Self::Integer => "could not parse integer".fmt(f),
            Self::Size => "could not parse size".fmt(f),
        }
    }
}

impl std::error::Error for CursorError {}

/// Read a byte from a cursor, moving the position forward by 1.
///
/// # Errors
///
/// If the cursor is at the end of the buffer, `CursorError::Incomplete`
/// is returned instead.
///
/// # Examples
/// ```
/// # use std::io::Cursor;
/// # use cursor::{byte, CursorError};
/// // Create some cursor over bytes [1,2]
/// let mut src: Cursor<&[u8]> = Cursor::new(&[1,2]);
///
/// // Read first byte ok
/// let first: Result<u8, CursorError> = byte(&mut src);
/// assert_eq!(first, Ok(1));
///
/// // Read second byte ok
/// let second: Result<u8, CursorError> = byte(&mut src);
/// assert_eq!(second, Ok(2));
///
/// // No third byte, error
/// let third: Result<u8, CursorError> = byte(&mut src);
/// assert_eq!(third, Err(CursorError::Incomplete));
/// ```
pub fn byte(src: &mut Cursor<&[u8]>) -> Result<u8, CursorError> {
    let pos = src.position();
    let byte = *src
        .get_ref()
        .get(pos as usize)
        .ok_or(CursorError::Incomplete)?;

    src.set_position(pos + 1);
    Ok(byte)
}

/// Read an ASCII-encoded, `\r\n`-terminated decimal size from a cursor,
/// advancing the position just past the `\n`.
///
/// # Errors
///
/// If there's no terminating `\r\n`, then `CursorError::Unterminated` is returned
/// and the cursor is not advanced.
///
/// Otherwise, if the bytes between the start of the cursor and the `\r\n`
/// do not represent the ASCII digit encoding of a `u64`, then `CursorError::Size`
/// is returned but the cursor is still advanced just past the `\n`.
///
/// # Examples
///
/// Reading from a slice successfully:
/// ```
/// # use std::io::Cursor;
/// # use cursor::{size, CursorError};
/// let mut src: Cursor<&[u8]> = Cursor::new("100\r\n".as_bytes());
///
/// let result: Result<u64, CursorError> = size(&mut src);
/// assert_eq!(result, Ok(100));
/// ```
/// Trying to read from a slice that's incomplete:
/// ```
/// # use std::io::Cursor;
/// # use cursor::{size, CursorError};
/// let mut src: Cursor<&[u8]> = Cursor::new("100".as_bytes());
///
/// let result: Result<u64, CursorError> = size(&mut src);
/// assert_eq!(result, Err(CursorError::Unterminated(3)));
/// ```
pub fn size(src: &mut Cursor<&[u8]>) -> Result<u64, CursorError> {
    let line = line(src)?;
    atoi::atoi(line).ok_or(CursorError::Size)
}

/// Read an ASCII-encoded, `\r\n`-terminated 64-bit signed integer from a cursor,
/// advancing the position just past the `\n`.
///
/// # Errors
///
/// If there's no terminating `\r\n`, then `CursorError::Unterminated` is returned
/// and the cursor is not advanced.
///
/// Otherwise, if the bytes between the start of the cursor and the `\r\n`
/// do not represent the ASCII digit encoding of a `i64`, then `CursorError::Integer`
/// is returned but the cursor is still advanced just past the `\n`.
///
/// # Examples
///
/// Reading from a slice successfully:
/// ```
/// # use std::io::Cursor;
/// # use cursor::{integer, CursorError};
/// let mut src: Cursor<&[u8]> = Cursor::new("100\r\n".as_bytes());
///
/// let result: Result<i64, CursorError> = integer(&mut src);
/// assert_eq!(result, Ok(100));
/// ```
/// Trying to read from a slice that's incomplete:
/// ```
/// # use std::io::Cursor;
/// # use cursor::{integer, CursorError};
/// let mut src: Cursor<&[u8]> = Cursor::new("100".as_bytes());
///
/// let result: Result<i64, CursorError> = integer(&mut src);
/// assert_eq!(result, Err(CursorError::Unterminated(3)));
/// ```
pub fn integer(src: &mut Cursor<&[u8]>) -> Result<i64, CursorError> {
    let line = line(src)?;
    atoi::atoi(line).ok_or(CursorError::Integer)
}

/// Read a `\r\n`-terminated line from a cursor, advancing the position
/// just past the `\n`.
///
/// # Errors
///
/// If there's no terminating `\r\n`, then `CursorError::Unterminated` is returned.
///
/// # Examples
///
/// Reading from a slice successfully:
/// ```
/// # use std::io::Cursor;
/// # use cursor::{line, CursorError};
/// let mut src: Cursor<&[u8]> = Cursor::new("Hello, world!\r\n".as_bytes());
///
/// let result: Result<&[u8], CursorError> = line(&mut src);
/// assert_eq!(result, Ok("Hello, world!".as_bytes()));
/// ```
/// Trying to read from a slice that's incomplete:
/// ```
/// # use std::io::Cursor;
/// # use cursor::{line, CursorError};
/// let mut src: Cursor<&[u8]> = Cursor::new("Hello, world!".as_bytes());
///
/// let result: Result<&[u8], CursorError> = line(&mut src);
/// assert_eq!(result, Err(CursorError::Unterminated(13)));
/// ```
pub fn line<'buf>(src: &mut Cursor<&'buf [u8]>) -> Result<&'buf [u8], CursorError> {
    let rem = src
        .get_ref()
        .get(src.position() as usize..)
        .expect("position in bounds");

    let index = rem
        .len()
        .checked_sub(1)
        .and_then(|end| (0..end).find(|&i| [rem[i], rem[i + 1]] == *b"\r\n"))
        .ok_or(CursorError::Unterminated(rem.len()))?;

    src.set_position(src.position() + index as u64 + 2);
    Ok(&rem[..index])
}

/// Read `len` bytes from a cursor, advancing the position to the next unread byte.
///
/// # Errors
///
/// If there aren't `len` bytes remaining, `CursorError::Incomplete` is returned
/// and the cursor is not advanced.
///
/// # Examples
///
/// Reading from a slice successfully:
/// ```
/// # use std::io::Cursor;
/// # use cursor::{slice, CursorError};
/// let mut src: Cursor<&[u8]> = Cursor::new("Hello, world!".as_bytes());
///
/// let result: Result<&[u8], CursorError> = slice(&mut src, 5);
/// assert_eq!(result, Ok("Hello".as_bytes()));
///
/// // Position is advanced past "Hello" now
/// let result: Result<&[u8], CursorError> = slice(&mut src, 5);
/// assert_eq!(result, Ok(", wor".as_bytes()));
/// ```
/// Trying to read from a slice that's incomplete:
/// ```
/// # use std::io::Cursor;
/// # use cursor::{slice, CursorError};
/// let mut src: Cursor<&[u8]> = Cursor::new("Hello, world!".as_bytes());
///
/// // Try to read too many bytes
/// let result: Result<&[u8], CursorError> = slice(&mut src, 20);
/// assert_eq!(result, Err(CursorError::Incomplete));
/// ```
pub fn slice<'buf>(src: &mut Cursor<&'buf [u8]>, len: u64) -> Result<&'buf [u8], CursorError> {
    let start = src.position();
    let end = start.checked_add(len).expect("overflow");

    let slice = src
        .get_ref()
        .get(start as usize..end as usize)
        .ok_or(CursorError::Incomplete)?;

    src.set_position(end);
    Ok(slice)
}
