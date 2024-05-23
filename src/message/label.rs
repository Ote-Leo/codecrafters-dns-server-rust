use std::{
    error::Error,
    fmt::{self, Display},
};

use bytes::BufMut;

/// An input sequence has a length greater than 255
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LabelError {
    MaxSizeReached(usize),
    IncompleteBuffer,
    FalseEncodedLength(u8),
}

impl Display for LabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LabelError::*;
        match self {
            MaxSizeReached(size) => {
                format!("input sequences must be of length smaller than 256, buf found '{size}'")
                    .fmt(f)
            }
            IncompleteBuffer => {
                "input buffer is either incomplete or doens't end in a null byte".fmt(f)
            }
            FalseEncodedLength(size) => format!(
                "a label is encoded with size '{size}', which more than the length of the \
                 input buffer"
            )
            .fmt(f),
        }
    }
}

impl Error for LabelError {}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Label(Vec<CharacterString>);

impl Label {
    /// Splits the input around b'.' to create a sequence of [`CharacterString`]s.
    pub fn parse(value: &[u8]) -> Result<Self, LabelError> {
        let mut buf = vec![];

        for string in value.split(|&e| e == b'.').into_iter() {
            let string = string.try_into()?;
            buf.push(string);
        }

        Ok(Self(buf))
    }

    /// Splits the input around '.' to create a sequence of [`CharacterString`]s.
    pub fn parse_str(value: &str) -> Result<Self, LabelError> {
        Self::parse(value.as_bytes())
    }

    /// The number of domain sotred inside the label
    pub fn domain_count(&self) -> usize {
        self.0.len()
    }

    /// The total number of bytes in a label
    pub fn len(&self) -> usize {
        let mut size = 0;
        for domain in self.0.iter() {
            size += domain.0.len();
        }
        size
    }

    /// The total number of bytes that should generate this label.
    pub fn original_len(&self) -> usize {
        let mut size = 0;
        for domain in self.0.iter() {
            // <L@length:u8><content:[u8;L]>
            size += 1 + domain.0.len();
        }
        size += 1; // null byte
        size
    }
}

impl TryFrom<&[u8]> for Label {
    type Error = LabelError;

    /// Handels raw binary input as a stream of <length><character-string>
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        use LabelError::*;
        let mut buf = value;
        let mut labels = vec![];

        loop {
            if buf.is_empty() {
                return Err(IncompleteBuffer);
            }

            match buf[0] {
                0 => break,
                length if buf.len() > length as usize => {
                    labels.push(buf[1..(length + 1) as usize].try_into()?);
                    buf = &buf[(length + 1) as usize..];
                }
                length => return Err(FalseEncodedLength(length)),
            }
        }

        Ok(Self(labels))
    }
}

impl From<Label> for Vec<u8> {
    fn from(value: Label) -> Self {
        let mut buf = vec![];
        for string in value.0.into_iter() {
            let bytes: Vec<u8> = string.into();
            buf.extend(bytes);
        }
        buf.put_u8(0);
        buf
    }
}

/// A single legnth octet followed by that number of characters.
///
/// CharacterStrings are treated as binary information, and can be up to 256 characters in length
/// (including the length octet)
#[derive(Debug, Clone, PartialEq, Eq)]
struct CharacterString(Vec<u8>);

impl TryFrom<&str> for CharacterString {
    type Error = LabelError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use LabelError::MaxSizeReached;
        Ok(match value.len() {
            length if length > 255 => return Err(MaxSizeReached(length)),
            _ => CharacterString(value.as_bytes().to_owned()),
        })
    }
}

impl TryFrom<&[u8]> for CharacterString {
    type Error = LabelError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        use LabelError::MaxSizeReached;
        Ok(match value.len() {
            length if length > 255 => return Err(MaxSizeReached(length)),
            _ => CharacterString(value.to_owned()),
        })
    }
}

impl From<CharacterString> for Vec<u8> {
    fn from(value: CharacterString) -> Self {
        let mut buf = vec![];
        buf.put_u8(value.0.len() as u8);
        buf.extend(value.0);
        buf
    }
}
