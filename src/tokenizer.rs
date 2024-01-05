mod emphasis;
mod html;
mod link;
mod list_item;

use once_cell::sync::Lazy;
use regex::Regex;

use self::{
    emphasis::tokenize_emphasis,
    html::tokenize_html,
    link::{tokenize_inline_link_dest, tokenize_link_label, tokenize_link_reference_definition},
    list_item::tokenize_list_item_type,
};
use crate::token::{Token, TokenType};

static ABSOLUTE_URI_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^<[a-zA-Z][a-zA-Z0-9+.\-]{1,31}:[\w!\?/\+\-_~=;\.,\*&@#\$%\(\)'\[\]]+>").unwrap()
});
static EMAIL_ADDRESS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^<([a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*)>$",).unwrap()
});

pub(crate) fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut buffer = String::new();

    let mut chars = input.chars().peekable();
    while let Some(&char) = chars.peek() {
        match char {
            '#' if is_head_of_line(&tokens, buffer.clone()) => {
                let mut level = 0;
                while chars.peek() == Some(&'#') {
                    level += 1;
                    chars.next();
                }

                let raw = "#".repeat(level);
                if chars.peek() != Some(&' ') {
                    buffer.push_str(raw.as_str());
                } else {
                    tokens.push(Token {
                        token_type: TokenType::ATXHeading((level as u8).into()),
                        raw,
                    });
                }
            }
            ' ' if is_head_of_line(&tokens, buffer.clone()) => {
                let mut count = 0;
                while chars.peek() == Some(&' ') {
                    count += 1;
                    chars.next();
                }

                if count >= 4 {
                    tokens.push(Token {
                        token_type: TokenType::IndentedCodeBlock,
                        raw: "    ".to_string(),
                    });
                    buffer.push_str(" ".repeat(count - 4).as_str());
                } else {
                    buffer.push_str(" ".repeat(count).as_str());
                }
            }
            ' ' => {
                let mut count = 0;
                while chars.peek() == Some(&' ') {
                    count += 1;
                    chars.next();
                }

                if count >= 2 && chars.peek() == Some(&'\n') {
                    chars.next(); // skip '\n'
                    if !buffer.is_empty() {
                        tokens.push(Token {
                            token_type: TokenType::Text,
                            raw: buffer.clone(),
                        });
                        buffer.clear();
                    }

                    tokens.push(Token {
                        token_type: TokenType::HardLineBreak,
                        raw: "  ".to_string(),
                    });
                } else {
                    buffer.push_str(" ".repeat(count).as_str());
                }
            }
            '`' => {
                let is_head_of_line = is_head_of_line(&tokens, buffer.clone());
                let mut count = 0;
                while chars.peek() == Some(&'`') {
                    count += 1;
                    chars.next();
                }

                if count >= 3 && is_head_of_line {
                    tokens.push(Token {
                        token_type: TokenType::FencedCodeBlock,
                        raw: "`".repeat(count),
                    });
                } else {
                    if !buffer.is_empty() {
                        tokens.push(Token {
                            token_type: TokenType::Text,
                            raw: buffer.clone(),
                        });
                        buffer.clear();
                    }

                    tokens.push(Token {
                        token_type: TokenType::CodeSpan,
                        raw: "`".repeat(count),
                    });
                }
            }
            '>' if is_head_of_line(&tokens, buffer.clone()) => {
                let mut count = 0;
                while chars.peek() == Some(&'>') {
                    count += 1;
                    chars.next();
                }

                tokens.push(Token {
                    token_type: TokenType::BlockQuote,
                    raw: ">".repeat(count),
                });
            }
            '<' => {
                let mut sub_buffer = String::new();
                while chars.peek() != Some(&'>')
                    && chars.peek() != Some(&'\n')
                    && chars.peek() != None
                {
                    sub_buffer.push(chars.next().unwrap());
                }
                if let Some(c) = chars.next() {
                    sub_buffer.push(c);
                }

                if ABSOLUTE_URI_REGEX.is_match(&sub_buffer)
                    || EMAIL_ADDRESS_REGEX.is_match(&sub_buffer)
                {
                    if !buffer.is_empty() {
                        tokens.push(Token {
                            token_type: TokenType::Text,
                            raw: buffer.clone(),
                        });
                        buffer.clear();
                    }

                    tokens.push(Token {
                        token_type: TokenType::AutoLink,
                        raw: sub_buffer,
                    });

                    continue;
                }

                tokenize_html(&mut tokens, &mut chars, &mut buffer, &mut sub_buffer);
            }
            '[' if is_head_of_line(&tokens, buffer.clone()) => {
                tokenize_link_reference_definition(&mut tokens, &mut chars)
            }
            '[' => {
                chars.next(); // skip '['
                if tokens
                    .last()
                    .map(|t| t.token_type == TokenType::LinkTextClosing)
                    == Some(true)
                {
                    match tokenize_link_label(&mut chars) {
                        Ok(label) => {
                            tokens.extend(vec![
                                Token {
                                    token_type: TokenType::LinkLabelMatchOpening,
                                    raw: "[".to_string(),
                                },
                                Token {
                                    token_type: TokenType::Text,
                                    raw: label,
                                },
                                Token {
                                    token_type: TokenType::LinkLabelMatchClosing,
                                    raw: "]".to_string(),
                                },
                            ]);
                            chars.next(); // skip ']'
                        }
                        Err(ts) => tokens.extend(ts),
                    }
                } else {
                    tokens.push(Token {
                        token_type: TokenType::LinkTextOpening,
                        raw: "[".to_string(),
                    });
                }
            }
            '!' => {
                chars.next(); // skip '!'
                if chars.peek() == Some(&'[') {
                    chars.next(); // skip '['
                    tokens.push(Token {
                        token_type: TokenType::ImageTextOpening,
                        raw: "![".to_string(),
                    });
                } else {
                    buffer.push('!');
                }
            }
            ']' => {
                if !buffer.is_empty() {
                    tokens.push(Token {
                        token_type: TokenType::Text,
                        raw: buffer.clone(),
                    });
                    buffer.clear();
                }

                chars.next(); // skip ']'
                tokens.push(Token {
                    token_type: TokenType::LinkTextClosing,
                    raw: "]".to_string(),
                });

                if chars.peek() == Some(&'(') {
                    chars.next(); // skip '('

                    match tokenize_inline_link_dest(&mut chars) {
                        Ok(ts) => tokens.extend(ts),
                        Err(t) => tokens.extend(tokenize(t.as_str())),
                    }
                }
            }
            '(' => {
                if buffer.is_empty()
                    && tokens
                        .last()
                        .map(|t| t.token_type == TokenType::LinkTextClosing)
                        == Some(true)
                {
                    chars.next(); // skip '('

                    match tokenize_inline_link_dest(&mut chars) {
                        Ok(ts) => tokens.extend(ts),
                        Err(t) => tokens.extend(tokenize(t.as_str())),
                    }
                } else {
                    buffer.push('(');
                    chars.next();
                }
            }
            '-' => {
                let is_head_of_line = is_head_of_line(&tokens, buffer.clone());
                let mut sub_buffer = String::new();
                while chars.peek() == Some(&'-') || chars.peek() == Some(&' ') {
                    sub_buffer.push(chars.next().unwrap());
                }

                if !buffer.is_empty() && buffer.chars().any(|c| c != ' ') || !is_head_of_line {
                    buffer.push_str(&sub_buffer);
                } else if sub_buffer.chars().filter(|&c| c == '-').count() >= 3
                    && (chars.peek() == Some(&'\n') || chars.peek() == None)
                {
                    tokens.push(Token {
                        token_type: TokenType::ThemanticBreak,
                        raw: sub_buffer,
                    });
                } else if chars.peek() == Some(&'\n') || chars.peek() == None {
                    tokens.push(Token {
                        token_type: TokenType::SetextHeading(2.into()),
                        raw: sub_buffer,
                    });
                } else if sub_buffer.chars().last() == Some(' ')
                    && sub_buffer.chars().filter(|&c| c == '-').count() == 1
                {
                    tokens.push(tokenize_list_item_type(&mut chars, sub_buffer));
                } else {
                    buffer.push_str(&sub_buffer);
                }
            }
            '*' => {
                let is_head_of_line = is_head_of_line(&tokens, buffer.clone());
                let mut sub_buffer = String::new();
                while chars.peek() == Some(&'*') || chars.peek() == Some(&' ') {
                    sub_buffer.push(chars.next().unwrap());
                }

                if is_head_of_line
                    && sub_buffer.chars().filter(|&c| c == '*').count() >= 3
                    && (chars.peek() == Some(&'\n') || chars.peek() == None)
                {
                    tokens.push(Token {
                        token_type: TokenType::ThemanticBreak,
                        raw: sub_buffer,
                    });
                } else if is_head_of_line
                    && sub_buffer.chars().last() == Some(' ')
                    && sub_buffer.chars().filter(|&c| c == '*').count() == 1
                {
                    tokens.push(tokenize_list_item_type(&mut chars, sub_buffer));
                } else {
                    tokenize_emphasis(&mut tokens, chars.peek(), &mut buffer, &mut sub_buffer);
                }
            }
            '_' => {
                let is_head_of_line = is_head_of_line(&tokens, buffer.clone());
                let mut sub_buffer = String::new();
                while chars.peek() == Some(&'_') || chars.peek() == Some(&' ') {
                    sub_buffer.push(chars.next().unwrap());
                }

                if is_head_of_line && sub_buffer.chars().filter(|&c| c == '_').count() >= 3 {
                    tokens.push(Token {
                        token_type: TokenType::ThemanticBreak,
                        raw: sub_buffer,
                    });
                } else {
                    tokenize_emphasis(&mut tokens, chars.peek(), &mut buffer, &mut sub_buffer);
                }
            }
            '+' => {
                let is_head_of_line = is_head_of_line(&tokens, buffer.clone());
                let mut sub_buffer = String::new();
                while chars.peek() == Some(&'+') || chars.peek() == Some(&' ') {
                    sub_buffer.push(chars.next().unwrap());
                }

                if is_head_of_line
                    && sub_buffer.chars().last() == Some(' ')
                    && sub_buffer.chars().filter(|&c| c == '+').count() == 1
                {
                    tokens.push(tokenize_list_item_type(&mut chars, sub_buffer));
                } else {
                    buffer.push_str(&sub_buffer);
                }
            }
            '=' => {
                let is_head_of_line = is_head_of_line(&tokens, buffer.clone());
                let mut sub_buffer = String::new();
                while chars.peek() == Some(&'=') || chars.peek() == Some(&' ') {
                    sub_buffer.push(chars.next().unwrap());
                }

                if is_head_of_line && (chars.peek() == Some(&'\n') || chars.peek() == None) {
                    tokens.push(Token {
                        token_type: TokenType::SetextHeading(1.into()),
                        raw: sub_buffer,
                    });
                } else {
                    buffer.push_str(&sub_buffer);
                }
            }
            '0'..='9' => {
                if !is_head_of_line(&tokens, buffer.clone()) {
                    buffer.push(char);
                    chars.next();

                    continue;
                };

                let mut sub_buffer = String::new();
                let mut digits = 0;
                while chars.peek().map(|c| c.is_ascii_digit()) == Some(true) && digits < 9 {
                    sub_buffer.push(chars.next().unwrap());
                    digits += 1;
                }

                let delimiter = if chars.peek() == Some(&'.') || chars.peek() == Some(&')') {
                    chars.next().unwrap()
                } else {
                    buffer.push_str(&sub_buffer);

                    continue;
                };

                if chars.peek() == Some(&' ') {
                    chars.next(); // skip ' '
                    tokens.push(Token {
                        token_type: TokenType::OrderedListItem,
                        raw: format!("{}{} ", sub_buffer, delimiter),
                    });
                } else {
                    buffer.push_str(&sub_buffer);
                    buffer.push(delimiter);
                }
            }
            '｜' => {
                if !buffer.is_empty() {
                    tokens.push(Token {
                        token_type: TokenType::Text,
                        raw: buffer.clone(),
                    });
                    buffer.clear();
                }

                tokens.push(Token {
                    token_type: TokenType::RubyTargetOpening,
                    raw: "｜".to_string(),
                });
                chars.next(); // skip '｜'
            }
            '《' => {
                if !buffer.is_empty() {
                    tokens.push(Token {
                        token_type: TokenType::Text,
                        raw: buffer.clone(),
                    });
                    buffer.clear();
                }

                tokens.push(Token {
                    token_type: TokenType::RubyTextOpening,
                    raw: "《".to_string(),
                });
                chars.next(); // skip '《'
            }
            '》' => {
                if !buffer.is_empty() {
                    tokens.push(Token {
                        token_type: TokenType::Text,
                        raw: buffer.clone(),
                    });
                    buffer.clear();
                }

                tokens.push(Token {
                    token_type: TokenType::RubyTextClosing,
                    raw: "》".to_string(),
                });
                chars.next(); // skip '》'
            }
            '\\' => {
                chars.next(); // skip '\\'
                if chars.peek() == Some(&'\n') {
                    chars.next(); // skip '\n'
                    if !buffer.is_empty() {
                        tokens.push(Token {
                            token_type: TokenType::Text,
                            raw: buffer.clone(),
                        });
                        buffer.clear();
                    }

                    tokens.push(Token {
                        token_type: TokenType::HardLineBreak,
                        raw: "\\".to_string(),
                    });
                } else {
                    buffer.push('\\');
                }
            }
            '\n' => {
                if is_head_of_line(&tokens, buffer.clone()) {
                    tokens.push(Token {
                        token_type: TokenType::BlankLine,
                        raw: "\n".to_string(),
                    });
                } else {
                    if !buffer.is_empty() {
                        tokens.push(Token {
                            token_type: TokenType::Text,
                            raw: buffer.clone(),
                        });
                        buffer.clear();
                    }
                    tokens.push(Token {
                        token_type: TokenType::SoftLineBreak,
                        raw: "\n".to_string(),
                    });
                }
                chars.next();
            }
            _ => {
                buffer.push(char);
                chars.next();
            }
        }
    }
    if !buffer.is_empty() {
        tokens.push(Token {
            token_type: TokenType::Text,
            raw: buffer.clone(),
        });
        buffer.clear();
    }

    tokens
}

fn is_head_of_line(tokens: &[Token], buffer: String) -> bool {
    if !tokens.is_empty() {
        let token_type = &tokens.last().unwrap().token_type;
        match token_type {
            TokenType::SoftLineBreak
            | TokenType::HardLineBreak
            | TokenType::BlockQuote
            | TokenType::BlankLine
            | TokenType::ThemanticBreak
            | TokenType::BulletListItem => {}
            _ => return false,
        }
    }

    if buffer.chars().all(|c| c == ' ') {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::{
        token::{DelimiterType, Token, TokenType},
        tokenizer::tokenize,
    };

    #[test]
    fn tokenize_heading() {
        // 見出し
        let input = "# Heading 1\n## Heading 2\n### Heading 3\n#### Heading 4\n##### Heading 5\n###### Heading 6";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::ATXHeading(1.into()),
                    raw: "#".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: " Heading 1".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::ATXHeading(2.into()),
                    raw: "##".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: " Heading 2".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::ATXHeading(3.into()),
                    raw: "###".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: " Heading 3".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::ATXHeading(4.into()),
                    raw: "####".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: " Heading 4".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::ATXHeading(5.into()),
                    raw: "#####".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: " Heading 5".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::ATXHeading(6.into()),
                    raw: "######".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: " Heading 6".to_string(),
                },
            ]
        );
    }

    #[test]
    fn tokenize_indent() {
        // インデント
        let input = " Hello, World!\n  Hello, World!\n   Hello, World!\n    Hello, World!\n     Hello, World!";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Text,
                    raw: " Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "  Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "   Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::IndentedCodeBlock,
                    raw: "    ".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::IndentedCodeBlock,
                    raw: "    ".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: " Hello, World!".to_string(),
                },
            ]
        );
    }

    #[test]
    fn tokenize_themantic_break() {
        // 水平線
        let input = "---\n***\n___";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::ThemanticBreak,
                    raw: "---".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::ThemanticBreak,
                    raw: "***".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::ThemanticBreak,
                    raw: "___".to_string(),
                },
            ]
        );
    }

    #[test]
    fn tokenize_list_item() {
        // リスト
        let input = "- Hello, World!\n* Hello, World!\n+ Hello, World!\n- [ ] Hello, World!\n- [x] Hello, World!\n1. Hello, World!";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::BulletListItem,
                    raw: "- ".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::BulletListItem,
                    raw: "* ".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::BulletListItem,
                    raw: "+ ".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::CheckListItem(false),
                    raw: "- [ ] ".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::CheckListItem(true),
                    raw: "- [x] ".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::OrderedListItem,
                    raw: "1. ".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
            ]
        );
    }

    #[test]
    fn tokenize_emphasis() {
        // 強調
        let input = "***Hello, **World!*";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Emphasis(DelimiterType::LeftFlanking),
                    raw: "***".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, ".to_string(),
                },
                Token {
                    token_type: TokenType::Emphasis(DelimiterType::LeftFlanking),
                    raw: "**".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "World!".to_string(),
                },
                Token {
                    token_type: TokenType::Emphasis(DelimiterType::RightFlanking),
                    raw: "*".to_string(),
                },
            ]
        );

        let input = "aaa_bbb_ccc\naaa*bbb*ccc";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Text,
                    raw: "aaa_bbb_ccc".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "aaa".to_string(),
                },
                Token {
                    token_type: TokenType::Emphasis(DelimiterType::Both),
                    raw: "*".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "bbb".to_string(),
                },
                Token {
                    token_type: TokenType::Emphasis(DelimiterType::Both),
                    raw: "*".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "ccc".to_string(),
                },
            ]
        );
    }

    #[test]
    fn tokenize_fenced_code_block() {
        // コードブロック
        let input = "```\nHello, World!\n```\naaa```bbb```ccc";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::FencedCodeBlock,
                    raw: "```".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::FencedCodeBlock,
                    raw: "```".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "aaa".to_string(),
                },
                Token {
                    token_type: TokenType::CodeSpan,
                    raw: "```".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "bbb".to_string(),
                },
                Token {
                    token_type: TokenType::CodeSpan,
                    raw: "```".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "ccc".to_string(),
                },
            ]
        );
    }

    #[test]
    fn tokenize_code_span() {
        // コードスパン
        let input = "`Hello, World!`\n``Hello, World!``";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::CodeSpan,
                    raw: "`".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::CodeSpan,
                    raw: "`".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::CodeSpan,
                    raw: "``".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::CodeSpan,
                    raw: "``".to_string(),
                }
            ]
        );
    }

    #[test]
    fn tokenize_auto_link() {
        // 自動リンク
        let input = "<https://example.com>\n<mailto:example@example>";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::AutoLink,
                    raw: "<https://example.com>".to_string(),
                },
                Token {
                    token_type: TokenType::SoftLineBreak,
                    raw: "\n".to_string(),
                },
                Token {
                    token_type: TokenType::AutoLink,
                    raw: "<mailto:example@example>".to_string(),
                },
            ]
        );
    }

    #[test]
    fn tokenize_html_block() {
        // HTMLブロック
        let input = "<form>\nHello, World!\n</form>\n\n<blockquote>\nHello, World!\n</blockquote>";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::HTMLBlock,
                    raw: "<form>\nHello, World!\n</form>".to_string(),
                },
                Token {
                    token_type: TokenType::HTMLBlock,
                    raw: "<blockquote>\nHello, World!\n</blockquote>".to_string(),
                },
            ]
        );
    }

    #[test]
    fn tokenize_inline_html() {
        // インラインHTML
        let input = "link: <a href=\"https://example.com\">Hello, World!</a>";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Text,
                    raw: "link: ".to_string(),
                },
                Token {
                    token_type: TokenType::RawHTML,
                    raw: "<a href=\"https://example.com\">".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::RawHTML,
                    raw: "</a>".to_string(),
                },
            ]
        );
    }

    #[test]
    fn tokenize_link() {
        // リンク
        let input = "[Hello, World!](https://example.com)";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::LinkTextOpening,
                    raw: "[".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::LinkTextClosing,
                    raw: "]".to_string(),
                },
                Token {
                    token_type: TokenType::LinkDestOpening,
                    raw: "(".to_string(),
                },
                Token {
                    token_type: TokenType::LinkDest,
                    raw: "https://example.com".to_string(),
                },
                Token {
                    token_type: TokenType::LinkDestClosing,
                    raw: ")".to_string(),
                },
            ]
        );
    }

    #[test]
    fn tokenize_image() {
        // 画像
        let input = "![Hello, World!](https://example.com)";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::ImageTextOpening,
                    raw: "![".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::LinkTextClosing,
                    raw: "]".to_string(),
                },
                Token {
                    token_type: TokenType::LinkDestOpening,
                    raw: "(".to_string(),
                },
                Token {
                    token_type: TokenType::LinkDest,
                    raw: "https://example.com".to_string(),
                },
                Token {
                    token_type: TokenType::LinkDestClosing,
                    raw: ")".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_block_quote() {
        // 引用
        let input = "> Hello, World!";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::BlockQuote,
                    raw: ">".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: " Hello, World!".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_ruby() {
        // ルビ
        let input = "｜Hello, World!《こんにちは、世界！》";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::RubyTargetOpening,
                    raw: "｜".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "Hello, World!".to_string(),
                },
                Token {
                    token_type: TokenType::RubyTextOpening,
                    raw: "《".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "こんにちは、世界！".to_string(),
                },
                Token {
                    token_type: TokenType::RubyTextClosing,
                    raw: "》".to_string(),
                },
            ]
        );

        let input = "こんにちは、世界《せかい》！";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Text,
                    raw: "こんにちは、世界".to_string(),
                },
                Token {
                    token_type: TokenType::RubyTextOpening,
                    raw: "《".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "せかい".to_string(),
                },
                Token {
                    token_type: TokenType::RubyTextClosing,
                    raw: "》".to_string(),
                },
                Token {
                    token_type: TokenType::Text,
                    raw: "！".to_string(),
                },
            ]
        );
    }
}
