#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Token {
    pub(crate) token_type: TokenType,
    pub(crate) raw: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenType {
    ThemanticBreak,
    ATXHeading(HeadingLevel),
    SetextHeading(HeadingLevel),
    IndentedCodeBlock,
    FencedCodeBlock,
    HTMLBlock,
    LinkReferenceDefinition(String, String, Option<String>),
    BlankLine,
    BlockQuote,
    BulletListItem,
    OrderedListItem,
    CheckListItem(bool),
    CodeSpan,
    Emphasis(DelimiterType),
    LinkTextOpening,
    LinkTextClosing,
    LinkDest,
    LinkTitle,
    LinkDestOpening,
    LinkDestClosing,
    ImageTextOpening,
    LinkLabelMatchOpening,
    LinkLabelMatchClosing,
    AutoLink,
    RawHTML,
    HardLineBreak,
    SoftLineBreak,
    Text,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum HeadingLevel {
    H1,
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DelimiterType {
    RightFlanking,
    LeftFlanking,
    Both,
}
