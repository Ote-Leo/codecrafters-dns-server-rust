mod header;
mod question;
mod type_class;

pub use header::*;
pub use question::*;
pub use type_class::*;

pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
}

impl Message {
    pub fn new(id: u16) -> Self {
        Self {
            header: HeaderBuilder::new().id(id).build(),
            questions: vec![],
        }
    }

    pub fn ask(&mut self, name: &str, typ: QuestionType, class: QuestionClass) {
        self.header.question_count += 1;

        let name = name.split('.').map(String::from).collect::<Vec<_>>();
        self.questions.push(Question { name, typ, class });
    }
}

impl From<Message> for Vec<u8> {
    fn from(value: Message) -> Self {
        let mut buf = vec![];

        let header: [u8; 12] = value.header.into();
        buf.extend_from_slice(&header);

        for question in value.questions.into_iter() {
            let question: Vec<u8> = question.into();
            buf.extend(question);
        }

        buf
    }
}
