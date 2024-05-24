//! The header contains the following fields:
//!
//! ```txt
//!                                     1  1  1  1  1  1
//!       0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                      ID                       |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                    QDCOUNT                    |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                    ANCOUNT                    |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                    NSCOUNT                    |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                    ARCOUNT                    |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! ```
use std::{
    error::Error,
    fmt::{self, Display},
};

use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// A random identifier is assigned to query packets. Response packets must reply with the same
    /// id. This is needed to differentiate responses due to the stateless nature of UDP.
    pub id: u16,

    /// Whether this message is a query or a response.
    pub typ: PacketType,

    /// The kind of query in this message.
    ///
    /// This value is set by the originator of a query and copied into the response.
    pub operation_code: OperationCode,

    /// Valid in responses, and specifies that the responding name server is an authority for the
    /// domain name in question seciton.
    pub authoritative_answer: bool,

    /// specifies that this message was truncated due to length greater than that permitted on the
    /// transmission channel.
    pub truncated_message: bool,

    /// Directs the name server to pursue the query recursively.
    ///
    /// This maybe set in a query and is copied into the response.
    pub recursion_desired: bool,

    /// Denotes whether recursive query support is available in the name server.
    pub recursion_available: bool,

    /// Response status code.
    pub response: Result<(), HeaderError>,

    /// The number of entries in the question section.
    pub question_count: u16,

    /// The number of entries in the answer section.
    pub answer_count: u16,

    /// The number of entries in the authority section.
    pub authority_count: u16,

    /// The number of entries in the addtional section.
    pub addtional_count: u16,
}

impl Default for Header {
    fn default() -> Self {
        HeaderBuilder::new().build()
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeaderBuilder {
    id: Option<u16>,
    typ: Option<PacketType>,
    operation_code: Option<OperationCode>,
    authoritative_answer: Option<bool>,
    truncated_message: Option<bool>,
    recursion_desired: Option<bool>,
    recursion_available: Option<bool>,
    response: Option<HeaderError>,
    question_count: Option<u16>,
    answer_count: Option<u16>,
    authority_count: Option<u16>,
    additional_count: Option<u16>,
}

impl HeaderBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn id(self, id: u16) -> Self {
        Self {
            id: Some(id),
            ..self
        }
    }

    pub fn typ(self, typ: PacketType) -> Self {
        Self {
            typ: Some(typ),
            ..self
        }
    }

    pub fn operation_code(self, operation_code: OperationCode) -> Self {
        Self {
            operation_code: Some(operation_code),
            ..self
        }
    }

    pub fn authoritative_answer(self, authoritative_answer: bool) -> Self {
        Self {
            authoritative_answer: Some(authoritative_answer),
            ..self
        }
    }

    pub fn truncated_message(self, truncated_message: bool) -> Self {
        Self {
            truncated_message: Some(truncated_message),
            ..self
        }
    }

    pub fn recursion_desired(self, recursion_desired: bool) -> Self {
        Self {
            recursion_desired: Some(recursion_desired),
            ..self
        }
    }

    pub fn recursion_available(self, recursion_available: bool) -> Self {
        Self {
            recursion_available: Some(recursion_available),
            ..self
        }
    }

    pub fn question_count(self, question_count: u16) -> Self {
        Self {
            question_count: Some(question_count),
            ..self
        }
    }

    pub fn response(self, response: Option<HeaderError>) -> Self {
        Self { response, ..self }
    }

    pub fn answer_count(self, answer_count: u16) -> Self {
        Self {
            answer_count: Some(answer_count),
            ..self
        }
    }

    pub fn authority_count(self, authority_count: u16) -> Self {
        Self {
            authority_count: Some(authority_count),
            ..self
        }
    }

    pub fn additional_count(self, additional_count: u16) -> Self {
        Self {
            additional_count: Some(additional_count),
            ..self
        }
    }

    pub fn build(self) -> Header {
        use OperationCode::StandardQuery;
        use PacketType::Response;

        Header {
            id: self.id.unwrap_or(1234),
            typ: self.typ.unwrap_or(Response),
            operation_code: self.operation_code.unwrap_or(StandardQuery),
            authoritative_answer: self.authoritative_answer.unwrap_or(false),
            truncated_message: self.truncated_message.unwrap_or(false),
            recursion_desired: self.recursion_desired.unwrap_or(false),
            recursion_available: self.recursion_available.unwrap_or(false),
            response: match self.response {
                Some(err) => Err(err),
                None => Ok(()),
            },
            question_count: self.question_count.unwrap_or(0),
            answer_count: self.answer_count.unwrap_or(0),
            authority_count: self.authority_count.unwrap_or(0),
            addtional_count: self.additional_count.unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    Query,
    Response,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OperationCode {
    StandardQuery,
    InverseQuery,
    StatusRequest,
    Reserved(u16),
}

impl From<OperationCode> for u16 {
    fn from(value: OperationCode) -> Self {
        use OperationCode::*;
        match value {
            StandardQuery => 0,
            InverseQuery => 1,
            StatusRequest => 2,
            Reserved(code) => code,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HeaderError {
    /// The name server was unable to interpret the query.
    Format = 1,

    /// The name server was unable to process this query due to a problem with the name server.
    ServerFailure,

    /// The domain name referenced in the query does not exist.
    Name,

    /// The name server does not support the request [`kind`] of query.
    ///
    /// [`kind`]: OperationCode
    NotImplemented,

    /// The name server refuses to perform the specified operation for policy reasons.
    Resfused,
}

impl Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use HeaderError::*;
        match self {
            Format => "The name server was unable to interpret the query.".fmt(f),
            ServerFailure => "The name server was unable to process this query due to a problem with the name server.".fmt(f),
            Name => "The domain name referenced in the query does not exist.".fmt(f),
            NotImplemented => "The name server does not support the request kind of query".fmt(f),
            Resfused => "The name server refuses to perform the specified operation for policy reasons" .fmt(f),
        }
    }
}

impl Error for HeaderError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderParseError {
    /// Parsing a header from a slice that isn't of size 12
    SliceSizeMismatch(usize),
    /// Using a reserved operation code (i.e. in range `(3..15)`)
    ReservedOperationCode(u8),
    /// Using a reserved response code (i.e. in range `(6..15)`)
    ReservedResponseCode(u8),
    /// `Z` flag is not set to zore
    ReservedZFlag(u8),
}

impl Display for HeaderParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use HeaderParseError::*;
        match self {
            SliceSizeMismatch(size) => {
                format!("a header must consist of 12 bytes, but found '{size}'").fmt(f)
            }
            ReservedOperationCode(code) => {
                format!("codes in 3..15 are reserved for future use, but found '{code}'").fmt(f)
            }
            ReservedResponseCode(code) => {
                format!("codes in 6..15 are reserved for future use, but found '{code}'").fmt(f)
            }
            ReservedZFlag(code) => {
                format!("z flag in header must be set to 0, but found '{code}'").fmt(f)
            }
        }
    }
}

impl Error for HeaderParseError {}

impl TryFrom<[u8; 12]> for Header {
    type Error = HeaderParseError;

    fn try_from(value: [u8; 12]) -> Result<Self, Self::Error> {
        use HeaderError::*;
        use HeaderParseError::*;
        use OperationCode::*;
        use PacketType::*;

        let mut buf = &value[..];
        let id = buf.get_u16();

        let flags = buf.get_u16();

        let typ = if (flags & (1 << 15)) != 0 {
            Response
        } else {
            Query
        };

        let operation_code = match ((flags & 0b0111_1000_0000_0000) >> 11) as u8 {
            0 => StandardQuery,
            1 => InverseQuery,
            2 => StatusRequest,
            code => {
                eprintln!("using fucking reserved operation code");
                Reserved(code as u16)
                // return Err(ReservedOperationCode(code)),
            }
        };

        let authoritative_answer = (flags & 0b0000_0100_0000_0000) != 0;
        let truncated_message = (flags & 0b0000_0010_0000_0000) != 0;
        let recursion_desired = (flags & 0b0000_0001_0000_0000) != 0;
        let recursion_available = (flags & 0b0000_0000_1000_0000) != 0;

        match ((flags & 0b0000_0000_0111_0000) >> 4) as u8 {
            0 => (),
            code => {
                eprintln!("the 'z' flag was set to {code}, but should've remained as zero");
                // return Err(ReservedZFlag(code)),
            }
        }

        let response = match (flags & 0b0000_0000_0000_1111) as u8 {
            0 => Ok(()),
            1 => Err(Format),
            2 => Err(ServerFailure),
            3 => Err(Name),
            4 => Err(NotImplemented),
            5 => Err(Resfused),
            code => return Err(ReservedResponseCode(code)),
        };

        let question_count = buf.get_u16();
        let answer_count = buf.get_u16();
        let authority_count = buf.get_u16();
        let addtional_count = buf.get_u16();

        Ok(Self {
            id,
            typ,
            operation_code,
            authoritative_answer,
            truncated_message,
            recursion_desired,
            recursion_available,
            response,
            question_count,
            answer_count,
            authority_count,
            addtional_count,
        })
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = HeaderParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        use HeaderParseError::SliceSizeMismatch;
        let buf: Result<[u8; 12], _> = value.try_into();

        match buf {
            Ok(buf) => buf.try_into(),
            Err(_) => Err(SliceSizeMismatch(value.len())),
        }
    }
}

impl From<Header> for [u8; 12] {
    fn from(header: Header) -> Self {
        let mut dst = [0u8; 12];
        let mut buf = &mut dst[..];

        buf.put_u16(header.id);

        let flags = {
            // Following the naming convension of <https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1>
            let qr = (header.typ as u16) << 15;
            let opcode = u16::from(header.operation_code) << 11;
            let aa = (header.authoritative_answer as u16) << 10;
            let tc = (header.truncated_message as u16) << 9;
            let rd = (header.recursion_desired as u16) << 8;
            let ra = (header.recursion_available as u16) << 7;
            let z = 0;
            let rcode = match header.response {
                Ok(_) => 0,
                Err(code) => code as u16,
            };

            qr | opcode | aa | tc | rd | ra | z | rcode
        };
        buf.put_u16(flags);

        buf.put_u16(header.question_count);
        buf.put_u16(header.answer_count);
        buf.put_u16(header.authority_count);
        buf.put_u16(header.addtional_count);

        dst
    }
}

impl From<Header> for Vec<u8> {
    fn from(header: Header) -> Self {
        let buf: [u8; 12] = header.into();
        buf.to_vec()
    }
}

#[cfg(test)]
mod parsing {
    use super::*;

    #[test]
    fn parse_compressed_packet_section_stage() {
        assert_eq!(
            Header::try_from([89, 81, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 22865,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([85, 132, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 21892,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 2,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );
    }

    #[test]
    fn parse_question_section_stage() {
        assert_eq!(
            Header::try_from([37, 206, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 9678,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([235, 215, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 60375,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );
    }

    #[test]
    fn parse_header_section_stage() {
        assert_eq!(
            Header::try_from([242, 139, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 62091,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([189, 13, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 48397,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([189, 13, 9, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 48397,
                typ: PacketType::Query,
                operation_code: OperationCode::InverseQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([189, 13, 17, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 48397,
                typ: PacketType::Query,
                operation_code: OperationCode::StatusRequest,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([189, 13, 25, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 48397,
                typ: PacketType::Query,
                operation_code: OperationCode::Reserved(3),
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([189, 13, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 48397,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: false,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([189, 13, 8, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 48397,
                typ: PacketType::Query,
                operation_code: OperationCode::InverseQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: false,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([189, 13, 16, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 48397,
                typ: PacketType::Query,
                operation_code: OperationCode::StatusRequest,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: false,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([189, 13, 24, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 48397,
                typ: PacketType::Query,
                operation_code: OperationCode::Reserved(3),
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: false,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );
    }

    #[test]
    fn write_answer_section_stage() {
        assert_eq!(
            Header::try_from([206, 83, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 52819,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([4, 210, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 1234,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );
    }

    #[test]
    fn write_question_section_stage() {
        assert_eq!(
            Header::try_from([176, 243, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 45299,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([4, 210, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 1234,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );
    }

    #[test]
    fn write_header_section_stage() {
        assert_eq!(
            Header::try_from([176, 132, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 45188,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );

        assert_eq!(
            Header::try_from([4, 210, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 1234,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );
    }

    #[test]
    fn setup_udp_server_stage() {
        assert_eq!(
            Header::try_from([147, 193, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
            Header {
                id: 37825,
                typ: PacketType::Query,
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                truncated_message: false,
                recursion_desired: true,
                recursion_available: false,
                response: Ok(()),
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                addtional_count: 0
            }
        );
    }
}
