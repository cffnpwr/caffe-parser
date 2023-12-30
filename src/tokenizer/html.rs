use std::{iter::Peekable, str::Chars};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::token::{Token, TokenType};

use super::is_head_of_line;

static HTMLBLOCK_TAG_BLOCK_START_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^<(pre|script|style|textarea)([\s>].*)?$").unwrap());
static HTMLBLOCK_DECLARATION_START_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^<![a-zA-Z].*").unwrap());
static HTMLBLOCK_TAG_PARAGRAPH_START_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^<(address|article|aside|base|basefont|blockquote|body|caption|center|col|colgroup|dd|details|dialog|dir|div|dl|dt|fieldset|figcaption|figure|footer|form|frame|frameset|h1|h2|h3|h4|h5|h6|head|header|hr|html|iframe|legend|li|link|main|menu|menuitem|nav|noframes|ol|optgroup|option|p|param|section|source|summary|table|tbody|td|tfoot|th|thead|title|tr|track|ul)([\s>(/>)].*)?$").unwrap()
});
static HTML_TAG_START_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^</?([a-zA-Z][a-zA-Z0-9\-]*?)(\s([a-zA-Z_:][a-zA-Z0-9_:.\-]*?)(=([^"'=<>`]+?|'(.+?)'|"(.+?)"))?)*?/?>"#).unwrap()
});

pub(super) fn tokenize_html(
    tokens: &mut Vec<Token>,
    chars: &mut Peekable<Chars>,
    prev_buffer: &mut String,
    buffer: &mut String,
) {
    let token = if is_head_of_line(&tokens, prev_buffer.clone())
        || prev_buffer.chars().all(|c| c == ' ')
    {
        let end_condition = if let Some(caps) = HTMLBLOCK_TAG_BLOCK_START_REGEX.captures(&buffer) {
            let tag_name = caps.get(1).unwrap().as_str();

            format!("</{}>", tag_name)
        } else if buffer.starts_with("<!--") {
            "-->".to_string()
        } else if buffer.starts_with("<?") {
            "?>".to_string()
        } else if HTMLBLOCK_DECLARATION_START_REGEX.is_match(&buffer) {
            ">".to_string()
        } else if buffer.starts_with("<![CDATA[") {
            "]]>".to_string()
        } else if HTMLBLOCK_TAG_PARAGRAPH_START_REGEX.is_match(&buffer)
            || HTML_TAG_START_REGEX.is_match(&buffer)
        {
            "\n\n".to_string()
        } else {
            prev_buffer.push_str(buffer);
            buffer.clear();

            return;
        };

        while let Some(&c) = chars.peek() {
            buffer.push(c);
            chars.next();

            if buffer.ends_with(&end_condition) {
                if buffer.ends_with("\n\n") {
                    buffer.truncate(buffer.len() - 2);
                }

                break;
            }
        }

        Token {
            token_type: TokenType::HTMLBlock,
            raw: buffer.clone(),
        }
    } else if HTML_TAG_START_REGEX.is_match(&buffer.to_ascii_lowercase()) {
        Token {
            token_type: TokenType::RawHTML,
            raw: buffer.clone(),
        }
    } else {
        prev_buffer.push_str(buffer);
        buffer.clear();

        return;
    };

    if !prev_buffer.is_empty() {
        tokens.push(Token {
            token_type: TokenType::Text,
            raw: prev_buffer.clone(),
        });
        prev_buffer.clear();
    }
    tokens.push(token);
}

#[cfg(test)]
mod test {
    use super::tokenize_html;
    use crate::token::{Token, TokenType};

    #[test]
    fn test_html_block_pre_tag_tokenize() {
        let mut tokens = vec![];
        let mut prev_buffer = "".to_string();
        let mut buffer = "<pre>\naaa\n</pre>".to_string();

        tokenize_html(
            &mut tokens,
            &mut prev_buffer.clone().chars().peekable(),
            &mut prev_buffer,
            &mut buffer,
        );
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::HTMLBlock,
                raw: "<pre>\naaa\n</pre>".to_string(),
            },]
        );
    }

    #[test]
    fn test_html_block_comment_tokenize() {
        let mut tokens = vec![];
        let mut prev_buffer = "".to_string();
        let mut buffer = "<!--this is comment.-->".to_string();

        tokenize_html(
            &mut tokens,
            &mut prev_buffer.clone().chars().peekable(),
            &mut prev_buffer,
            &mut buffer,
        );
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::HTMLBlock,
                raw: "<!--this is comment.-->".to_string(),
            },]
        );
    }

    #[test]
    fn test_html_block_processing_instruction_tokenize() {
        let mut tokens = vec![];
        let mut prev_buffer = "".to_string();
        let mut buffer = r#"<?php\necho("this is PHP source code.")\n?>"#.to_string();

        tokenize_html(
            &mut tokens,
            &mut prev_buffer.clone().chars().peekable(),
            &mut prev_buffer,
            &mut buffer,
        );
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::HTMLBlock,
                raw: r#"<?php\necho("this is PHP source code.")\n?>"#.to_string(),
            },]
        );
    }

    #[test]
    fn test_html_block_declaration_tokenize() {
        let mut tokens = vec![];
        let mut prev_buffer = "".to_string();
        let mut buffer = "<!doctype html>".to_string();

        tokenize_html(
            &mut tokens,
            &mut prev_buffer.clone().chars().peekable(),
            &mut prev_buffer,
            &mut buffer,
        );
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::HTMLBlock,
                raw: "<!doctype html>".to_string(),
            },]
        );
    }

    #[test]
    fn test_html_block_cdata_tokenize() {
        let mut tokens = vec![];
        let mut prev_buffer = "".to_string();
        let mut buffer = "<![CDATA[<sender>John Smith</sender>]]>".to_string();

        tokenize_html(
            &mut tokens,
            &mut prev_buffer.clone().chars().peekable(),
            &mut prev_buffer,
            &mut buffer,
        );
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::HTMLBlock,
                raw: "<![CDATA[<sender>John Smith</sender>]]>".to_string(),
            },]
        );
    }

    #[test]
    fn test_html_block_paragraph_tag_tokenize() {
        let mut tokens = vec![];
        let mut prev_buffer = "".to_string();
        let mut buffer = "<form>\naaa\n</form>".to_string();

        tokenize_html(
            &mut tokens,
            &mut prev_buffer.clone().chars().peekable(),
            &mut prev_buffer,
            &mut buffer,
        );
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::HTMLBlock,
                raw: "<form>\naaa\n</form>".to_string(),
            },]
        );

        let mut tokens = vec![];
        let mut prev_buffer = "".to_string();
        let mut buffer = "<form action=\"https://example.com\">aaa</form>".to_string();

        tokenize_html(
            &mut tokens,
            &mut prev_buffer.clone().chars().peekable(),
            &mut prev_buffer,
            &mut buffer,
        );
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::HTMLBlock,
                raw: "<form action=\"https://example.com\">aaa</form>".to_string(),
            },]
        );
    }

    #[test]
    fn test_html_block_any_tag_tokenize() {
        let mut tokens = vec![];
        let mut prev_buffer = "".to_string();
        let mut buffer = "<any-tag>aaa</any-tag>".to_string();

        tokenize_html(
            &mut tokens,
            &mut prev_buffer.clone().chars().peekable(),
            &mut prev_buffer,
            &mut buffer,
        );
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::HTMLBlock,
                raw: "<any-tag>aaa</any-tag>".to_string(),
            },]
        );

        let mut tokens = vec![];
        let mut prev_buffer = "".to_string();
        let mut buffer = "<any-tag any-attr=\"any-value\">aaa</any-tag>".to_string();

        tokenize_html(
            &mut tokens,
            &mut prev_buffer.clone().chars().peekable(),
            &mut prev_buffer,
            &mut buffer,
        );
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::HTMLBlock,
                raw: "<any-tag any-attr=\"any-value\">aaa</any-tag>".to_string(),
            },]
        );
    }

    #[test]
    fn test_html_inline_tokenize() {
        let mut tokens = vec![];
        let mut prev_buffer = "hogehoge".to_string();
        let mut buffer = "<any-tag>aaa</any-tag>".to_string();

        tokenize_html(
            &mut tokens,
            &mut prev_buffer.clone().chars().peekable(),
            &mut prev_buffer,
            &mut buffer,
        );

        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Text,
                    raw: "hogehoge".to_string(),
                },
                Token {
                    token_type: TokenType::RawHTML,
                    raw: "<any-tag>aaa</any-tag>".to_string(),
                },
            ]
        );
    }
}
