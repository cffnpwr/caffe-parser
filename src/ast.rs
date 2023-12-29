use crate::token::HeadingLevel;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ASTNode {
    // Block
    Heading {
        level: HeadingLevel,
        children: Vec<ASTNode>,
    },
    Paragraph(Vec<ASTNode>),
    BlockQuote(Vec<ASTNode>),
    CodeBlock {
        language: String,
        text: String,
    },
    List {
        list_type: ListType,
        children: Vec<ListItem>,
    },
    HorizontalRule,

    // Inline
    Bold(Vec<ASTNode>),
    Italic(Vec<ASTNode>),
    Code(String),
    Link {
        href: String,
        title: Option<String>,
        children: Vec<ASTNode>,
    },
    Image {
        href: String,
        title: Option<String>,
        alt: String,
    },

    // Text
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ListType {
    Ordered,
    Unordered,
    Checked,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ListItem {
    pub(crate) checked: bool,
    pub(crate) children: Vec<ASTNode>,
}
