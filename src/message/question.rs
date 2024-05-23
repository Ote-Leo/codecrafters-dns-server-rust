//! The question section contains fields that describe a question to a name server.  These fields
//! are a query type (QTYPE), a query class (QCLASS), and a query domain name (QNAME)./! The
//! [`question section`] is used to carry the [`Question`] in most queries, i.e., the
//! parameters that define what is being asked.  The section contains [`QDCOUNT`] (usually 1)
//! entries, each of the following format:
//!
//!                                     1  1  1  1  1  1
//!       0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                                               |
//!     /                     QNAME                     /
//!     /                                               /
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                     QTYPE                     |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                     QCLASS                    |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!
//! [`QDCOUNT`]: super::header::Header::question_count
//! [`question section`]: super::Message::questions

use std::{
    error::Error,
    fmt::{self, Display},
    str,
};

use bytes::{Buf, BufMut};

use super::type_class::{QuestionClass, QuestionType, UnregisteredClass, UnregisteredType};

pub struct Question {
    /// A domain name represented as a sequence of labels, where each label consists of a length
    /// octet followed by that number of octets.  The domain name terminates with the zero length
    /// octet for the null label of the root.  Note that this field may be an odd number of octets;
    /// no padding is used.
    pub name: Vec<String>,

    /// The values for this field include all codes valid for a TYPE field, together with some more
    /// general codes which can match more than one type of RR.
    pub typ: QuestionType,

    /// The class of the query.
    pub class: QuestionClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionParseError {
    IncompleteBuffer,
    FalseEncodedLength,
    NoneUtf8EncodedLabel,
    InvalidType(UnregisteredType),
    InvalidClass(UnregisteredClass),
    MissingTypeAndClass,
    MissingClass,
}

impl From<UnregisteredType> for QuestionParseError {
    fn from(value: UnregisteredType) -> Self {
        Self::InvalidType(value)
    }
}

impl From<UnregisteredClass> for QuestionParseError {
    fn from(value: UnregisteredClass) -> Self {
        Self::InvalidClass(value)
    }
}

impl Display for QuestionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use QuestionParseError::*;
        match self {
            IncompleteBuffer => "buffer got consumed while reading labels".fmt(f),
            FalseEncodedLength => {
                "encoded length is greater than the remaining of the buffer".fmt(f)
            }
            NoneUtf8EncodedLabel => "label isn't properly encoded".fmt(f),
            InvalidType(err) => err.fmt(f),
            InvalidClass(err) => err.fmt(f),
            MissingTypeAndClass => "buffer doesn't contain a type and a class".fmt(f),
            MissingClass => "buffer doesn't contain a class".fmt(f),
        }
    }
}

impl Error for QuestionParseError {}

impl TryFrom<&[u8]> for Question {
    type Error = QuestionParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        use QuestionParseError::*;

        let mut buf = value;
        let mut name = vec![];

        // reading labels
        loop {
            if buf.is_empty() {
                return Err(IncompleteBuffer);
            }

            match buf[0] {
                0 => break,
                length if buf.len() > length as usize => {
                    match str::from_utf8(&buf[1..(length + 1) as usize]) {
                        Ok(label) => name.push(label.to_string()),
                        Err(_) => return Err(NoneUtf8EncodedLabel),
                    }

                    buf = &buf[(length + 1) as usize..];
                }
                _ => return Err(FalseEncodedLength),
            }
        }

        // reading type and class
        let typ;
        let class;
        match buf.len() {
            0 | 1 => return Err(MissingTypeAndClass),
            2 | 3 => return Err(MissingClass),
            _ => {
                typ = buf.get_u16().try_into()?;
                class = buf.get_u16().try_into()?;
            }
        }

        Ok(Self { name, typ, class })
    }
}

impl From<Question> for Vec<u8> {
    fn from(value: Question) -> Self {
        let mut buf = vec![];

        // writing labels
        for label in value.name {
            let length = label.len();

            // shouldn't happend as it ensured by the parsing step, but you never know
            assert!(
                length < 256,
                "label '{label}' consists of more that 255 bytes!",
            );

            buf.put_u8(label.len() as u8);
            buf.put_slice(label.as_bytes());
        }
        buf.put_u8(0);

        // writing type and class
        buf.put_u16(value.typ as u16);
        buf.put_u16(value.class as u16);

        buf
    }
}
