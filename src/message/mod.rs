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
pub mod type_class;

pub use header::*;
pub use label::*;
pub use question::*;
pub use type_class::*;

pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
}

impl Message {
    /// Create a new [`Message`] with the given `id`
    pub fn new(id: u16) -> Self {
        Self {
            header: HeaderBuilder::new().id(id).build(),
            questions: vec![],
        }
    }

    /// Add a [`Question`] to the message
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
}

impl From<Message> for Vec<u8> {
    fn from(value: Message) -> Self {
        let mut buf = vec![];

        let header: [u8; 12] = value.header.into();
        buf.extend_from_slice(&header);

        for question in value.questions.into_iter() {
            buf.extend::<Vec<_>>(question.into());
        }

        buf
    }
}
