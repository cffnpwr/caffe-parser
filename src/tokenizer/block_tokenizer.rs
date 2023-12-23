use regex::Regex;

use crate::token::{ListType, Token};
use once_cell::sync::Lazy;

static HEADING_PREFIX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\s]{0,3}#{1,6}").unwrap());
static BLOCK_QUOTE_PREFIX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\s]{0,3}>").unwrap());
static CODE_BLOCK_PREFIX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\s]{0,3}(`{3,}|~{3,})(.*?)$").unwrap());
static CODE_BLOCK_SUFFIX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(`{3,}|~{3,})$").unwrap());
static LIST_PREFIX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\s]{0,3}([*+-]|\d\.)\s").unwrap());
static HORIZONTAL_RULE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\s]{0,3}([-\s\t]{3,}|[_\s\t]{3,}|[*\s\t]{3,})$").unwrap());

pub(super) fn block_tokenize(input: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut buffer = String::new();

    let mut lines = input.lines().peekable();
    'line: while let Some(line) = lines.next() {
        if buffer.is_empty() && line.is_empty() {
            continue;
        }

        // 見出し
        if HEADING_PREFIX_REGEX.is_match(line) {
            let mut level = 0;
            let mut chars = line.chars().peekable();
            while let Some('#') = chars.peek() {
                chars.next();
                level += 1;

                // 見出しレベルが6を超えたら、それは見出しではない
                if level > 6 {
                    buffer += line;

                    continue 'line;
                }
            }
            if !buffer.is_empty() {
                tokens.push(Token::Paragraph(buffer.clone()));
                buffer.clear();
            }

            // 空白は無視する
            while let Some(' ') = chars.peek() {
                chars.next();
            }

            tokens.push(Token::Heading(level.into(), chars.collect()));

            continue 'line;
        }

        // ブロック引用
        if BLOCK_QUOTE_PREFIX_REGEX.is_match(line) {
            if !buffer.is_empty() {
                tokens.push(Token::Paragraph(buffer.clone()));
                buffer.clear();
            }

            buffer += BLOCK_QUOTE_PREFIX_REGEX.replace_all(line, "").trim();
            while let Some(line) = lines.next() {
                if line.is_empty()
                    || HEADING_PREFIX_REGEX.is_match(line)
                    || LIST_PREFIX_REGEX.is_match(line)
                {
                    continue;
                } else {
                    buffer += "\n";
                    buffer += BLOCK_QUOTE_PREFIX_REGEX.replace_all(line, "").trim();
                }
            }

            tokens.push(Token::BlockQuote(buffer.clone()));
            buffer.clear();

            continue 'line;
        }

        // コードブロック(インデント)
        if line.starts_with("    ") {
            if !buffer.is_empty() {
                tokens.push(Token::Paragraph(buffer.clone()));
                buffer.clear();
            }

            buffer += line.get(4..).unwrap();
            while let Some(line) = lines.next() {
                if line.is_empty() || !line.starts_with("    ") {
                    if buffer.ends_with('\n') {
                        buffer.pop();
                    }

                    continue;
                } else {
                    buffer += "\n";
                    buffer += line.get(4..).unwrap();
                }
            }

            tokens.push(Token::CodeBlock("".to_string(), buffer.clone()));
            buffer.clear();

            continue 'line;
        }

        // コードブロック(フェンス)
        if CODE_BLOCK_PREFIX_REGEX.is_match(line) {
            if !buffer.is_empty() {
                tokens.push(Token::Paragraph(buffer.clone()));
                buffer.clear();
            }

            let caps = CODE_BLOCK_PREFIX_REGEX.captures(line).unwrap();
            let fence = caps.get(1).unwrap().as_str().chars().next().unwrap();
            let prefix_count = caps.get(1).unwrap().len();
            let lang = caps.get(2).unwrap().as_str().trim().to_string();

            while let Some(line) = lines.next() {
                if CODE_BLOCK_SUFFIX_REGEX.is_match(line)
                    && line.starts_with(fence)
                    && CODE_BLOCK_SUFFIX_REGEX
                        .captures(line)
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .len()
                        >= prefix_count
                {
                    if buffer.ends_with('\n') {
                        buffer.pop();
                    }

                    continue;
                } else {
                    buffer += line;
                    buffer += "\n";
                }
            }

            tokens.push(Token::CodeBlock(lang, buffer.clone()));
            buffer.clear();

            continue 'line;
        }

        // リスト
        if LIST_PREFIX_REGEX.is_match(line) {
            if !buffer.is_empty() {
                tokens.push(Token::Paragraph(buffer.clone()));
                buffer.clear();
            }

            let mut list_items = vec![];
            let list_text = LIST_PREFIX_REGEX.replace_all(line, "");
            let list_text = list_text.trim().to_string();
            let cap = LIST_PREFIX_REGEX.captures(line).unwrap();
            let maker = cap.get(1).unwrap().as_str();
            let mut list_type = match maker {
                "*" | "+" | "-" => {
                    if list_text.starts_with("[ ] ") || list_text.starts_with("[x] ") {
                        ListType::Checked
                    } else {
                        ListType::Unordered
                    }
                }
                _ => ListType::Ordered,
            };
            list_items.push(list_text);

            while let Some(line) = lines.next() {
                if line.is_empty() {
                    continue;
                } else if LIST_PREFIX_REGEX.is_match(line) {
                    let list_text = LIST_PREFIX_REGEX.replace_all(line, "");
                    let list_text = list_text.trim().to_string();
                    let cap = LIST_PREFIX_REGEX.captures(line).unwrap();
                    let maker = cap.get(1).unwrap().as_str();
                    match maker {
                        "*" | "+" | "-" => {
                            if list_text.starts_with("[ ] ") || list_text.starts_with("[x] ") {
                                if list_type != ListType::Checked {
                                    tokens.push(Token::List(list_type, list_items.clone()));
                                    list_items.clear();

                                    list_type = ListType::Checked;
                                }
                            } else {
                                if list_type != ListType::Unordered {
                                    tokens.push(Token::List(list_type, list_items.clone()));
                                    list_items.clear();

                                    list_type = ListType::Unordered;
                                }
                            }
                        }
                        _ => {
                            if list_type != ListType::Ordered {
                                tokens.push(Token::List(list_type, list_items.clone()));
                                list_items.clear();

                                list_type = ListType::Ordered;
                            }
                        }
                    }

                    buffer += list_text.as_str();
                    list_items.push(buffer.clone());
                    buffer.clear();
                } else {
                    buffer += line;
                    buffer += "\n";
                }
            }

            tokens.push(Token::List(list_type, list_items.clone()));
            buffer.clear();

            continue 'line;
        }

        // Horizontal Rule
        if HORIZONTAL_RULE_REGEX.is_match(line) {
            if !buffer.is_empty() {
                tokens.push(Token::Paragraph(buffer.clone()));
                buffer.clear();
            }

            tokens.push(Token::HorizontalRule);
            buffer.clear();

            continue 'line;
        }

        // 段落
        if line.is_empty() && !buffer.is_empty() {
            if buffer.ends_with('\n') {
                buffer.pop();
            }

            tokens.push(Token::Paragraph(buffer.clone()));
            buffer.clear();

            continue 'line;
        }
        buffer += line;
        buffer += "\n";
    }

    if buffer.ends_with('\n') {
        buffer.pop();
    }
    if !buffer.is_empty() {
        tokens.push(Token::Paragraph(buffer.clone()));
        buffer.clear();
    }

    return tokens;
}

#[cfg(test)]
mod tests {
    //     use crate::{token::Token, tokenizer::tokenize};

    //     #[test]
    //     fn tokenize_inline_toekn() {
    //         let heading = "# Heading";
    //         let test = "Hello, world!";
    //         let bold = "**Bold**";
    //         let italic = "*Italic*";
    //         let code = "`Code`";
    //         let link = "[Link](https://example.com)";
    //     }

    use crate::{
        token::{HeadingLevel, Token},
        tokenizer::block_tokenizer::block_tokenize,
    };

    #[test]
    fn test_block_tokenize() {
        let heading = "# Heading";
        let block_quote = "> hello\n>world\n  > hogehoge\n >fuga";
        let indent_code_block = "    fn main() {\n        println!(\"Hello, world!\");\n    }";
        let fenced_code_block = "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n";
        let unordered_list = "* Hello, world!\n* Hello, world!\n* Hello, world!";
        let ordered_list = "1. Hello, world!\n2. Hello, world!\n3. Hello, world!";
        let checked_list = "- [ ] Hello, world!\n- [x] Hello, world!\n- [ ] Hello, world!";
        let horizontal_rule = "---";
        let paragraph = "Hello, world!\nHello, world!\n\nHello, world!\n\n####### Hello, world!";

        assert_eq!(
            block_tokenize(heading),
            vec![Token::Heading(HeadingLevel::H1, "Heading".into()),]
        );
        assert_eq!(
            block_tokenize(block_quote),
            vec![Token::BlockQuote("hello\nworld\nhogehoge\nfuga".into()),]
        );
        assert_eq!(
            block_tokenize(indent_code_block),
            vec![Token::CodeBlock(
                "".to_string(),
                "fn main() {\n    println!(\"Hello, world!\");\n}".into()
            ),]
        );
        assert_eq!(
            block_tokenize(fenced_code_block),
            vec![Token::CodeBlock(
                "rust".to_string(),
                "fn main() {\n    println!(\"Hello, world!\");\n}".into()
            ),]
        );
        assert_eq!(
            block_tokenize(unordered_list),
            vec![Token::List(
                crate::token::ListType::Unordered,
                vec![
                    "Hello, world!".into(),
                    "Hello, world!".into(),
                    "Hello, world!".into(),
                ]
            ),]
        );
        assert_eq!(
            block_tokenize(ordered_list),
            vec![Token::List(
                crate::token::ListType::Ordered,
                vec![
                    "Hello, world!".into(),
                    "Hello, world!".into(),
                    "Hello, world!".into(),
                ]
            ),]
        );
        assert_eq!(
            block_tokenize(checked_list),
            vec![Token::List(
                crate::token::ListType::Checked,
                vec![
                    "[ ] Hello, world!".into(),
                    "[x] Hello, world!".into(),
                    "[ ] Hello, world!".into(),
                ]
            ),]
        );
        assert_eq!(block_tokenize(horizontal_rule), vec![Token::HorizontalRule]);
        assert_eq!(
            block_tokenize(paragraph),
            vec![
                Token::Paragraph("Hello, world!\nHello, world!".into()),
                Token::Paragraph("Hello, world!".into()),
                Token::Paragraph("####### Hello, world!".into()),
            ]
        );
    }
}
