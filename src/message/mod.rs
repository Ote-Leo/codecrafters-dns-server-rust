//! All communications inside of the domain protocol are carried in a single format called a
//! [`message`].  The top level format of message is divided into 5 sections (some of which are
//! empty in certain cases) shown below:
//!
//! ```txt
//!     +---------------------+
//!     |        Header       |
//!     +---------------------+
//!     |       Question      | the question for the name server
//!     +---------------------+
//!     |        Answer       | RRs answering the question
//!     +---------------------+
//!     |      Authority      | RRs pointing toward an authority
//!     +---------------------+
//!     |      Additional     | RRs holding additional information
//!     +---------------------+
//! ```
//!
//! The [`header section`][mhd] is always present.  The header includes fields that specify which
//! of the remaining sections are present, and also specify whether the message is a [`query`] or a
//! [`response`], a standard query or some other [`opcode`], etc.
//!
//! The names of the sections after the header are derived from their use in [`standard queries`].
//! The [`question section`] contains fields that describe a question to a name server.  These
//! fields are a query type ([`QTYPE`]), a query class ([`QCLASS`]), and a query domain name
//! ([`QNAME`]). The last three sections have the same format: a possibly empty list of
//! concatenated resource records (RRs). The answer section contains RRs that answer the question;
//! the authority section contains RRs that point toward an authoritative name server; the
//! additional records section contains RRs which relate to the query, but are not strictly answers
//! for the question.
//!
//! [mhd]: Message::header
//! [`message`]: Message
//! [`query`]: header::PacketType::Query
//! [`response`]: header::PacketType::Response
//! [`opcode`]: header::OperationCode
//! [`standard queries`]: header::OperationCode::StandardQuery
//! [`question section`]: Message::questions
//!
//! [`QNAME`]: question::Question::name
//! [`QTYPE`]: question::Question::typ
//! [`QCLASS`]: question::Question::class
pub mod header;
pub mod label;
pub mod question;
pub mod resource;
pub mod type_class;

use std::{
    error::Error,
    fmt::{self, Display},
};

pub use header::*;
pub use label::*;
pub use question::*;
pub use resource::*;
pub use type_class::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authorities: Vec<ResourceRecord>,
    pub additionals: Vec<ResourceRecord>,
}

impl Message {
    /// Create a new [`Message`] with the given `id`
    pub fn new(id: u16) -> Self {
        Self {
            header: HeaderBuilder::new().id(id).build(),
            questions: vec![],
            answers: vec![],
            authorities: vec![],
            additionals: vec![],
        }
    }

    /// Set the message type to [`query`][PacketType::Query]
    pub fn query(&mut self) {
        self.header.typ = PacketType::Query;
    }

    /// Set the message type to [`response`][PacketType::Response]
    pub fn respond(&mut self) {
        self.header.typ = PacketType::Response;
    }

    /// Add a [`Question`] to the message [`questions`][Self::questions]
    ///
    /// The `name` is split around `.` and stored as a sequence of labels
    pub fn ask(
        &mut self,
        name: &str,
        typ: QuestionType,
        class: QuestionClass,
    ) -> Result<(), LabelError> {
        self.header.question_count += 1;

        let name = Label::parse_str(name)?;
        self.questions.push(Question { name, typ, class });
        Ok(())
    }

    /// Add a [`ResourceRecord`] to the message [`answers`][Self::answers]
    pub fn answer(&mut self, rr: ResourceRecord) {
        self.header.answer_count += 1;
        self.answers.push(rr);
    }

    /// Add a [`ResourceRecord`] to the message [`authorities`][Self::authorities]
    pub fn authorize(&mut self, rr: ResourceRecord) {
        self.header.authority_count += 1;
        self.authorities.push(rr);
    }

    /// Add a [`ResourceRecord`] to the message [`additionals`][Self::additionals]
    pub fn add(&mut self, rr: ResourceRecord) {
        self.header.addtional_count += 1;
        self.additionals.push(rr);
    }
}

impl From<Message> for Vec<u8> {
    fn from(value: Message) -> Self {
        let mut buf = vec![];

        let header: [u8; 12] = value.header.into();
        buf.extend_from_slice(&header);

        for question in value.questions.into_iter() {
            buf.extend::<Vec<_>>(question.into());
        }

        for answer in value.answers.into_iter() {
            buf.extend::<Vec<_>>(answer.into());
        }

        for authority in value.authorities.into_iter() {
            buf.extend::<Vec<_>>(authority.into());
        }

        for additional in value.additionals.into_iter() {
            buf.extend::<Vec<_>>(additional.into());
        }

        buf
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageParseError {
    /// Messages are at least 12 bytes
    ShortBuffer,
    Header(HeaderParseError),
    Resource(ResourceRecordError),
    Question(QuestionParseError),
}

impl From<HeaderParseError> for MessageParseError {
    fn from(value: HeaderParseError) -> Self {
        Self::Header(value)
    }
}

impl From<ResourceRecordError> for MessageParseError {
    fn from(value: ResourceRecordError) -> Self {
        Self::Resource(value)
    }
}

impl From<QuestionParseError> for MessageParseError {
    fn from(value: QuestionParseError) -> Self {
        Self::Question(value)
    }
}

impl Display for MessageParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageParseError::ShortBuffer => "messages must be at least 12 bytes".fmt(f),
            MessageParseError::Header(err) => err.fmt(f),
            MessageParseError::Resource(err) => err.fmt(f),
            MessageParseError::Question(err) => err.fmt(f),
        }
    }
}

impl Error for MessageParseError {}

impl TryFrom<&[u8]> for Message {
    type Error = MessageParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 12 {
            return Err(MessageParseError::ShortBuffer);
        }
        let header: Header = value[..12].try_into()?;

        let mut buf = &value[12..];
        eprintln!("header: {header:?}");
        eprintln!("buf: {buf:?}");

        eprintln!("parsing questions");
        let mut questions = vec![];
        for _ in 0..header.question_count {
            let (mut question, offset) = parse_question(buf)?;

            expand_label(&mut question.name, value);

            eprintln!("question: {question:?}");
            questions.push(question);
            buf = &buf[offset..];
            eprintln!("buf: {buf:?}");
        }

        eprintln!("parsing answers");
        let mut answers = vec![];
        for _ in 0..header.answer_count {
            let (mut answer, offset) = parse_resource_record(buf)?;

            expand_label(&mut answer.name, value);

            eprintln!("answer: {answer:?}");
            answers.push(answer);
            buf = &buf[offset..];
            eprintln!("buf: {buf:?}");
        }

        eprintln!("parsing authorities");
        let mut authorities = vec![];
        for _ in 0..header.authority_count {
            let (mut authority, offset) = parse_resource_record(buf)?;

            expand_label(&mut authority.name, value);

            eprintln!("authority: {authority:?}");
            authorities.push(authority);
            buf = &buf[offset..];
            eprintln!("buf: {buf:?}");
        }

        eprintln!("parsing additionals");
        let mut additionals = vec![];
        for _ in 0..header.addtional_count {
            let (mut additional, offset) = parse_resource_record(buf)?;

            expand_label(&mut additional.name, value);

            eprintln!("additional: {additional:?}");
            additionals.push(additional);
            buf = &buf[offset..];
            eprintln!("buf: {buf:?}");
        }

        Ok(Self {
            header,
            questions,
            answers,
            authorities,
            additionals,
        })
    }
}

fn expand_label(label: &mut Label, buf: &[u8]) {
    let last = label.0.pop();
    if let Some(CharacterString::Compressed(offset)) = last {
        eprintln!("decompressing label at index {offset}");
        let (expanded_label, _) =
            parse_label(&buf[offset as usize..]).expect("false compressed offset");
        label.0.extend(expanded_label.0);
        expand_label(label, buf) // in-case that the expanded label is also compressed
    } else if let Some(last) = last {
        label.0.push(last);
    }
}
