use super::tokenize;
use crate::token::{Token, TokenType};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{iter::Peekable, str::Chars};

static LINK_LABEL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(([^\[\]\s]|\\\[|\\\])|([^\[\]]|\\\[|\\\]){1,999})").unwrap());
static LINK_DEST_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(<([^<>]|\\\>|\\\<)+>|[^\s\x00-\x1F\x7F]+)").unwrap());

pub(super) fn tokenize_link_reference_definition(
    tokens: &mut Vec<Token>,
    chars: &mut Peekable<Chars>,
) {
    chars.next(); // skip '['
    let link_label = match tokenize_link_label(chars) {
        Ok(t) => t,
        Err(ts) => {
            tokens.extend(ts);

            return;
        }
    };
    chars.next(); // skip ']'

    if chars.peek() != Some(&':') {
        tokens.push(Token {
            token_type: TokenType::LinkTextOpening,
            raw: "[".to_string(),
        });
        tokens.extend(tokenize(&link_label));
        tokens.push(Token {
            token_type: TokenType::LinkTextClosing,
            raw: "]".to_string(),
        });

        return;
    }

    chars.next(); // skip ':'

    let mut is_newlined = false;
    let mut spaces_between_label_and_dest = String::new();
    while chars.peek() == Some(&' ') || !is_newlined && chars.peek() == Some(&'\n') {
        let char = chars.next().unwrap();
        if char == '\n' && !is_newlined {
            is_newlined = true;
        }
        spaces_between_label_and_dest.push(char);
    }
    if is_newlined && chars.peek() == Some(&'\n') {
        tokens.push(Token {
            token_type: TokenType::LinkTextOpening,
            raw: "[".to_string(),
        });
        tokens.extend(tokenize(&link_label));
        tokens.push(Token {
            token_type: TokenType::LinkTextClosing,
            raw: "]".to_string(),
        });
        tokens.extend(tokenize(
            format!(":{}", spaces_between_label_and_dest).as_str(),
        ));

        return;
    }

    let link_dest = match tokenize_link_dest(chars) {
        Ok(t) => t,
        Err(t) => {
            tokens.push(Token {
                token_type: TokenType::LinkTextOpening,
                raw: "[".to_string(),
            });
            tokens.extend(tokenize(&link_label));
            tokens.push(Token {
                token_type: TokenType::LinkTextClosing,
                raw: "]".to_string(),
            });
            tokens.extend(tokenize(
                format!(":{}{}", spaces_between_label_and_dest, t).as_str(),
            ));

            return;
        }
    };

    let mut is_newlined = false;
    let mut spaces_between_dest_and_title = String::new();
    while chars.peek() == Some(&' ') || !is_newlined && chars.peek() == Some(&'\n') {
        let char = chars.next().unwrap();
        if char == '\n' && !is_newlined {
            is_newlined = true;
        }
        spaces_between_dest_and_title.push(char);
    }
    if is_newlined && chars.peek() == Some(&'\n') {
        tokens.push(Token {
            token_type: TokenType::LinkTextOpening,
            raw: "[".to_string(),
        });
        tokens.extend(tokenize(&link_label));
        tokens.push(Token {
            token_type: TokenType::LinkTextClosing,
            raw: "]".to_string(),
        });
        tokens.extend(tokenize(
            format!(
                ":{}{}{}",
                spaces_between_label_and_dest, link_dest, spaces_between_dest_and_title
            )
            .as_str(),
        ));

        return;
    }

    let title = match chars.peek() {
        Some(&'"') | Some(&'\'') | Some(&'(') => {
            let quote = chars.next().unwrap(); // skip quote

            let title = tokenize_link_title(chars, quote);
            match title {
                Ok(title) => {
                    chars.next(); // skip quote

                    Some(format!("{1}{}{1}", title, quote))
                }
                Err(t) => {
                    tokens.push(Token {
                        token_type: TokenType::LinkTextOpening,
                        raw: "[".to_string(),
                    });
                    tokens.extend(tokenize(&link_label));
                    tokens.push(Token {
                        token_type: TokenType::LinkTextClosing,
                        raw: "]".to_string(),
                    });
                    tokens.extend(tokenize(
                        format!(
                            ":{}{}{}{}{}",
                            spaces_between_label_and_dest,
                            link_dest,
                            spaces_between_dest_and_title,
                            quote,
                            t
                        )
                        .as_str(),
                    ));

                    return;
                }
            }
        }
        _ => None,
    };

    tokens.push(Token {
        token_type: TokenType::LinkReferenceDefinition(
            link_label.clone(),
            link_dest.clone(),
            title
                .clone()
                .map(|title| title[1..title.len() - 1].to_string()),
        ),
        raw: format!(
            "[{}]:{}{}{}{}",
            link_label,
            spaces_between_label_and_dest,
            link_dest,
            spaces_between_dest_and_title,
            title.unwrap_or_default()
        ),
    })
}

pub(super) fn tokenize_inline_link_dest(chars: &mut Peekable<Chars>) -> Result<Vec<Token>, String> {
    let mut tokens = vec![];

    let mut is_newlined = false;
    let mut spaces_between_label_and_dest = String::new();
    while chars.peek() == Some(&' ') || !is_newlined && chars.peek() == Some(&'\n') {
        let char = chars.next().unwrap();
        if char == '\n' && !is_newlined {
            is_newlined = true;
        }
        spaces_between_label_and_dest.push(char);
    }
    if is_newlined && chars.peek() == Some(&'\n') {
        return Err(format!("{}", spaces_between_label_and_dest));
    }

    let link_dest = match tokenize_link_dest(chars) {
        Ok(dest) => dest,
        Err(t) => {
            return Err(format!("{}", t));
        }
    };

    let mut is_newlined = false;
    let mut spaces_between_dest_and_title = String::new();
    while chars.peek() == Some(&' ') || !is_newlined && chars.peek() == Some(&'\n') {
        let char = chars.next().unwrap();
        if char == '\n' && !is_newlined {
            is_newlined = true;
        }
        spaces_between_dest_and_title.push(char);
    }
    if is_newlined && chars.peek() == Some(&'\n') {
        return Err(format!("{}", spaces_between_dest_and_title));
    }

    let title = match chars.peek() {
        Some(&'"') | Some(&'\'') | Some(&'(') => {
            let quote = chars.next().unwrap(); // skip quote

            let title = tokenize_link_title(chars, quote);
            match title {
                Ok(title) => {
                    chars.next(); // skip quote

                    Some(title)
                }
                Err(t) => {
                    return Err(format!(
                        "{}{}{}{}{}",
                        spaces_between_label_and_dest,
                        link_dest,
                        spaces_between_dest_and_title,
                        quote,
                        t
                    ))
                }
            }
        }
        _ => None,
    };

    let mut is_newlined = false;
    let mut spaces_between_title_and_close = String::new();
    while chars.peek() == Some(&' ') || !is_newlined && chars.peek() == Some(&'\n') {
        let char = chars.next().unwrap();
        if char == '\n' && !is_newlined {
            is_newlined = true;
        }
        spaces_between_title_and_close.push(char);
    }
    if is_newlined && chars.peek() == Some(&'\n') {
        return Err(format!("{}", spaces_between_title_and_close));
    }

    if chars.peek() == Some(&')') {
        chars.next(); // skip ')'
        tokens.extend(vec![
            Token {
                token_type: TokenType::LinkDestOpening,
                raw: "(".to_string(),
            },
            Token {
                token_type: TokenType::LinkDest,
                raw: link_dest,
            },
        ]);
        if let Some(title) = title {
            tokens.push(Token {
                token_type: TokenType::LinkTitle,
                raw: title,
            });
        }
        tokens.push(Token {
            token_type: TokenType::LinkDestClosing,
            raw: ")".to_string(),
        });
    } else {
        return Err(format!(
            "{}{}{}{}{}",
            spaces_between_label_and_dest,
            link_dest,
            spaces_between_dest_and_title,
            title.unwrap_or_default(),
            spaces_between_title_and_close
        ));
    }

    Ok(tokens)
}

pub(super) fn tokenize_link_label(chars: &mut Peekable<Chars>) -> Result<String, Vec<Token>> {
    let mut link_label = String::new();
    while chars.peek() != Some(&']') && chars.peek() != None {
        let mut char = chars.next().unwrap();
        if char == '\\' && chars.peek().is_some() {
            link_label.push(char);
            char = chars.next().unwrap();
        } else if char == '\n' {
            link_label.push(char);

            if chars.peek() == Some(&'\n') {
                let mut tokens = vec![];
                tokens.push(Token {
                    token_type: TokenType::LinkTextOpening,
                    raw: "[".to_string(),
                });
                tokens.extend(tokenize(&link_label));

                return Err(tokens);
            }
        } else if char == '[' {
            let mut tokens = vec![];
            tokens.push(Token {
                token_type: TokenType::LinkTextOpening,
                raw: "[".to_string(),
            });
            tokens.extend(tokenize(&link_label));
            tokens.push(Token {
                token_type: TokenType::LinkTextOpening,
                raw: "[".to_string(),
            });

            return Err(tokens);
        }

        link_label.push(char);
    }

    if !LINK_LABEL_REGEX.is_match(&link_label) || chars.peek() == None {
        let mut tokens = vec![];
        tokens.push(Token {
            token_type: TokenType::LinkTextOpening,
            raw: "[".to_string(),
        });
        tokens.extend(tokenize(&link_label));

        return Err(tokens);
    }

    Ok(link_label)
}

pub(super) fn tokenize_link_dest(chars: &mut Peekable<Chars>) -> Result<String, String> {
    let allow_space = chars.peek() == Some(&'<');
    let mut link_dest = if allow_space {
        chars.next();
        "<".to_string()
    } else {
        String::new()
    };
    let mut parentheses = 0;
    while let Some(&char) = chars.peek() {
        match allow_space {
            true => match char {
                '>' | '<' => {
                    link_dest.push(chars.next().unwrap());

                    break;
                }
                '\n' => return Err(link_dest),
                '\\' => {
                    link_dest.push(char);
                    chars.next();

                    if chars.peek().is_some() {
                        link_dest.push(chars.next().unwrap());
                    }

                    continue;
                }
                _ => link_dest.push(chars.next().unwrap()),
            },
            false => match char {
                '(' => {
                    parentheses += 1;
                    link_dest.push(chars.next().unwrap());
                }
                ')' if parentheses == 0 => break,
                ')' => {
                    parentheses -= 1;
                    link_dest.push(chars.next().unwrap());
                }
                '\\' => {
                    link_dest.push(char);
                    chars.next();

                    if chars.peek().is_some() {
                        link_dest.push(chars.next().unwrap());
                    }

                    continue;
                }
                _ if char.is_ascii_control() || char.is_ascii_whitespace() => break,
                _ => link_dest.push(chars.next().unwrap()),
            },
        }
    }
    if !LINK_DEST_REGEX.is_match(&link_dest) {
        return Err(link_dest);
    }

    Ok(link_dest)
}

pub(super) fn tokenize_link_title(
    chars: &mut Peekable<Chars>,
    quote: char,
) -> Result<String, String> {
    let mut link_title = String::new();
    while let Some(&char) = chars.peek() {
        match char {
            '\\' => {
                link_title.push(char);
                chars.next();

                if chars.peek().is_some() {
                    link_title.push(chars.next().unwrap());
                }

                continue;
            }
            '\n' => {
                link_title.push(char);
                chars.next();

                if chars.peek() == Some(&'\n') {
                    return Err(link_title);
                }
            }
            _ if char == quote => break,
            _ => link_title.push(chars.next().unwrap()),
        }
    }

    if link_title.is_empty() {
        return Err(link_title);
    }

    Ok(link_title)
}

#[cfg(test)]
mod tests {
    use crate::{
        token::{Token, TokenType},
        tokenizer::link::tokenize_link_reference_definition,
    };

    #[test]
    fn test_tokenize_link_reference_definition() {
        let mut tokens = vec![];
        let mut chars = "[link]: https://example.com".chars().peekable();
        tokenize_link_reference_definition(&mut tokens, &mut chars);
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::LinkReferenceDefinition(
                    "link".to_string(),
                    "https://example.com".to_string(),
                    None
                ),
                raw: "[link]: https://example.com".to_string(),
            }]
        );

        let mut tokens = vec![];
        let mut chars = "[link]: https://example.com \"title\"".chars().peekable();
        tokenize_link_reference_definition(&mut tokens, &mut chars);
        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::LinkReferenceDefinition(
                    "link".to_string(),
                    "https://example.com".to_string(),
                    Some("title".to_string())
                ),
                raw: "[link]: https://example.com \"title\"".to_string(),
            }]
        );
    }
}
