mod emphasis;

use crate::token::{Token, TokenType};

use self::emphasis::tokenize_emphasis;

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
                    tokens.push(Token {
                        token_type: TokenType::BulletListItem,
                        raw: sub_buffer,
                    });
                } else {
                    buffer.push_str(&sub_buffer);
                }
            }
            '*' => {
                let is_head_of_line =
                    is_head_of_line(&tokens, buffer.clone()) || buffer.chars().all(|c| c == ' ');
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
                    tokens.push(Token {
                        token_type: TokenType::BulletListItem,
                        raw: sub_buffer,
                    });
                } else {
                    tokenize_emphasis(&mut tokens, chars.peek(), &mut buffer, &mut sub_buffer);
                }
            }
            '_' => {
                let is_head_of_line =
                    is_head_of_line(&tokens, buffer.clone()) || buffer.chars().all(|c| c == ' ');
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
                let is_head_of_line =
                    is_head_of_line(&tokens, buffer.clone()) || buffer.chars().all(|c| c == ' ');
                let mut sub_buffer = String::new();
                while chars.peek() == Some(&'+') || chars.peek() == Some(&' ') {
                    sub_buffer.push(chars.next().unwrap());
                }

                if is_head_of_line
                    && sub_buffer.chars().last() == Some(' ')
                    && sub_buffer.chars().filter(|&c| c == '+').count() == 1
                {
                    tokens.push(Token {
                        token_type: TokenType::BulletListItem,
                        raw: sub_buffer,
                    });
                } else {
                    buffer.push_str(&sub_buffer);
                }
            }
            '=' => {
                let is_head_of_line =
                    is_head_of_line(&tokens, buffer.clone()) || buffer.chars().all(|c| c == ' ');
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
    if !buffer.is_empty() {
        return false;
    } else if !tokens.is_empty() {
        let token_type = &tokens.last().unwrap().token_type;
        if token_type != &TokenType::SoftLineBreak
            && token_type != &TokenType::HardLineBreak
            && token_type != &TokenType::BlankLine
        {
            return false;
        }
    }

    true
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
        let input = "- Hello, World!\n* Hello, World!\n+ Hello, World!";
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
}
