use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum Token {
    // Block
    Heading(HeadingLevel, String),
    Paragraph(String),
    BlockQuote(String),
    CodeBlock(String, String),
    List(ListType, Vec<String>),
    HorizontalRule,

    // Inline
    Bold(String),
    Italic(String),
    Code(String),
    Link(String, Option<String>, String),
    Image(String, Option<String>, String),

    // Test
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum ListType {
    Ordered,
    Unordered,
    Checked,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum HeadingLevel {
    H1 = 1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl From<u8> for HeadingLevel {
    fn from(level: u8) -> Self {
        match level {
            1 => HeadingLevel::H1,
            2 => HeadingLevel::H2,
            3 => HeadingLevel::H3,
            4 => HeadingLevel::H4,
            5 => HeadingLevel::H5,
            6 => HeadingLevel::H6,
            _ => unreachable!(),
        }
    }
}
