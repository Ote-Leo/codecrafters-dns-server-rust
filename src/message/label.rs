use std::{
    error::Error,
    fmt::{self, Display},
};

use bytes::{Buf, BufMut};

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
pub struct Label(pub Vec<CharacterString>);

impl Label {
    /// Splits the input around b'.' to create a sequence of [`CharacterString`]s.
    pub fn parse(value: &[u8]) -> Result<Self, LabelError> {
        let mut buf = vec![];

        for string in value.split(|&e| e == b'.') {
            match string.len() {
                0 => return Err(LabelError::IncompleteBuffer),
                length if length > 255 => return Err(LabelError::MaxSizeReached(length)),
                _ => buf.push(CharacterString::String(string.to_owned())),
            }
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
}

pub fn parse_label(value: &[u8]) -> Result<(Label, usize), LabelError> {
    use LabelError::*;
    let mut buf = value;
    let mut labels = vec![];
    let mut offset = 0;

    loop {
        if buf.is_empty() {
            return Err(IncompleteBuffer);
        }

        match buf[0] {
            0 => break,
            _ => {
                let (string, len) = parse_character_string(buf)?;
                buf = &buf[len..];
                offset += len;
                if let CharacterString::Compressed(_) = string {
                    labels.push(string);
                    return Ok((Label(labels), offset));
                }
                labels.push(string);
            }
        }
    }

    Ok((Label(labels), offset + 1))
}

impl TryFrom<&[u8]> for Label {
    type Error = LabelError;

    /// Handels raw binary input as a stream of <length><character-string>
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_label(value).map(|t| t.0)
    }
}

impl From<Label> for Vec<u8> {
    fn from(value: Label) -> Self {
        let mut buf = vec![];

        let last = value.0.last().map(Clone::clone);

        for string in value.0.into_iter() {
            let bytes: Vec<u8> = string.into();
            buf.extend(bytes);
        }

        if let Some(CharacterString::Compressed(_)) = last {
            return buf;
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
pub enum CharacterString {
    String(Vec<u8>),
    Compressed(u16),
}

impl TryFrom<&str> for CharacterString {
    type Error = LabelError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

impl TryFrom<&[u8]> for CharacterString {
    type Error = LabelError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_character_string(value).map(|t| t.0)
    }
}

pub fn parse_character_string(value: &[u8]) -> Result<(CharacterString, usize), LabelError> {
    use CharacterString::*;
    use LabelError::*;
    let mut buf = value;
    Ok(match value.len() {
        0 => return Err(IncompleteBuffer),
        length if length > 255 => return Err(MaxSizeReached(length)),
        length => match value[0] as usize {
            count if (count & 0b1100_0000) >> 6 == 3 => {
                let offset = buf.get_u16() ^ 0b1100_0000_0000_0000;
                (Compressed(offset), 2)
            }
            count if count < length => (String(value[1..count + 1].to_owned()), count + 1),
            count => return Err(FalseEncodedLength(count as u8)),
        },
    })
}

impl From<CharacterString> for Vec<u8> {
    fn from(value: CharacterString) -> Self {
        use CharacterString::*;
        let mut buf = vec![];

        match value {
            String(string) => {
                buf.put_u8(string.len() as u8);
                buf.extend(string);
            }
            Compressed(offset) => buf.put_u16(offset | 0b1100_0000_0000_0000),
        }
        buf
    }
}
