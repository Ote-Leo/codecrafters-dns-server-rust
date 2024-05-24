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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Label {
    /// Raw [`CharacterString`] sequence
    Sequence(Vec<CharacterString>),
    /// An offset for the label
    Compressed(u16),
}

impl Label {
    /// Splits the input around b'.' to create a sequence of [`CharacterString`]s.
    pub fn parse(value: &[u8]) -> Result<Self, LabelError> {
        let mut buf = vec![];

        for string in value.split(|&e| e == b'.') {
            match string.len() {
                0 => return Err(LabelError::IncompleteBuffer),
                length if length > 255 => return Err(LabelError::MaxSizeReached(length)),
                _ => buf.push(CharacterString(string.to_owned())),
            }
        }

        Ok(Self::Sequence(buf))
    }

    /// Splits the input around '.' to create a sequence of [`CharacterString`]s.
    pub fn parse_str(value: &str) -> Result<Self, LabelError> {
        Self::parse(value.as_bytes())
    }
}

pub fn parse_label(value: &[u8]) -> Result<(Label, usize), LabelError> {
    use LabelError::*;
    let mut buf = value;

    let res = {
        if buf.is_empty() {
            return Err(IncompleteBuffer);
        }

        match buf[0] {
            length if (length & 0b1100_0000) >> 6 == 3 => {
                let offset = buf.get_u16() ^ 0b1100_0000_0000_0000;
                (Label::Compressed(offset), buf.remaining())
            }
            _ => {
                let mut labels = vec![];
                let mut offset = 0;

                loop {
                    match buf[0] {
                        0 => break,
                        _ => {
                            let (string, len) = parse_character_string(buf)?;
                            labels.push(string);
                            offset += len;
                            buf = &buf[len..];
                        }
                    }

                    if buf.is_empty() {
                        return Err(IncompleteBuffer);
                    }
                }
                offset += 1;

                (Label::Sequence(labels), offset)
            }
        }
    };

    Ok(res)
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
        use Label::*;
        let mut buf = vec![];

        match value {
            Sequence(seq) => {
                for string in seq.into_iter() {
                    let bytes: Vec<u8> = string.into();
                    buf.extend(bytes);
                }
                buf.put_u8(0);
            }
            Compressed(offset) => buf.put_u16(offset | 0b1100_0000_0000_0000),
        }

        buf
    }
}

/// A single legnth octet followed by that number of characters.
///
/// CharacterStrings are treated as binary information, and can be up to 256 characters in length
/// (including the length octet)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterString(pub Vec<u8>);

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
    use LabelError::*;
    Ok(match value.len() {
        0 => return Err(IncompleteBuffer),
        length if length > 255 => return Err(MaxSizeReached(length)),
        length => match value[0] as usize {
            count if count < length => (CharacterString(value[1..count + 1].to_owned()), count + 1),
            count => return Err(FalseEncodedLength(count as u8)),
        },
    })
}

impl From<CharacterString> for Vec<u8> {
    fn from(value: CharacterString) -> Self {
        let mut buf = vec![];
        buf.put_u8(value.0.len() as u8);
        buf.extend(value.0);
        buf
    }
}
