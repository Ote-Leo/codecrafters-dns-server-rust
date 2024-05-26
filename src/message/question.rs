//! The [`question section`] is used to carry the [`Question`] in most queries, i.e., the
//! parameters that define what is being asked.  The section contains [`QDCOUNT`] (usually 1)
//! entries, each of the following format:
//!
//! ```txt`
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
//! ```
//!
//! [`QDCOUNT`]: super::header::Header::question_count
//! [`question section`]: super::Message::questions

use std::{
    error::Error,
    fmt::{self, Display},
};

use bytes::{Buf, BufMut};

use crate::message::parse_label;

use super::{
    label::{Label, LabelError},
    type_class::{QuestionClass, QuestionType, UnregisteredClass, UnregisteredType},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// A domain name represented as a sequence of labels, where each label consists of a length
    /// octet followed by that number of octets.  The domain name terminates with the zero length
    /// octet for the null label of the root.  Note that this field may be an odd number of octets;
    /// no padding is used.
    pub name: Label,

    /// The values for this field include all codes valid for a TYPE field, together with some more
    /// general codes which can match more than one type of RR.
    pub typ: QuestionType,

    /// The class of the query.
    pub class: QuestionClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionParseError {
    Label(LabelError),
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

impl From<LabelError> for QuestionParseError {
    fn from(value: LabelError) -> Self {
        Self::Label(value)
    }
}

impl Display for QuestionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use QuestionParseError::*;
        match self {
            Label(err) => err.fmt(f),
            InvalidType(err) => err.fmt(f),
            InvalidClass(err) => err.fmt(f),
            MissingTypeAndClass => "buffer doesn't contain a type and a class".fmt(f),
            MissingClass => "buffer doesn't contain a class".fmt(f),
        }
    }
}

impl Error for QuestionParseError {}

pub fn parse_question(value: &[u8]) -> Result<(Question, usize), QuestionParseError> {
    use QuestionParseError::{MissingClass, MissingTypeAndClass};

    let mut buf = value;
    let mut question_offset;

    // reading labels
    let (name, offset) = parse_label(buf)?;
    buf = &buf[offset..];
    question_offset = offset;

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
    question_offset += 4;

    Ok((Question { name, typ, class }, question_offset))
}

impl TryFrom<&[u8]> for Question {
    type Error = QuestionParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_question(value).map(|t| t.0)
    }
}

impl From<Question> for Vec<u8> {
    fn from(value: Question) -> Self {
        let mut buf = vec![];

        // writing labels
        buf.extend::<Vec<_>>(value.name.into());

        // writing type and class
        buf.put_u16(value.typ as u16);
        buf.put_u16(value.class as u16);

        buf
    }
}
