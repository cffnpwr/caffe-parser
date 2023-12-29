use crate::token::{DelimiterType, Token};
use crate::tokenizer::TokenType;
use crate::util::is_unicode_punctuation;

pub(super) fn tokenize_emphasis(
    tokens: &mut Vec<Token>,
    next_char: Option<&char>,
    prev_buffer: &mut String,
    buffer: &mut String,
) {
    let last_char = if !prev_buffer.is_empty() {
        prev_buffer.chars().last()
    } else if !tokens.is_empty() {
        tokens.last().unwrap().raw.chars().last()
    } else {
        None
    };
    let is_left_flanking_delimiter_run = if buffer.ends_with(" ") {
        false
    } else {
        match next_char {
            Some(&' ') | Some(&'\n') | Some(&'\t') | None => false,
            Some(&c) if is_unicode_punctuation(c) => {
                let splitted = buffer.rsplit_once(' ');
                if splitted.is_some() {
                    true
                } else {
                    let last_char = if !prev_buffer.is_empty() {
                        prev_buffer.chars().last().unwrap()
                    } else if !tokens.is_empty() {
                        tokens.last().unwrap().raw.chars().last().unwrap()
                    } else {
                        '\n'
                    };
                    if last_char == ' '
                        || last_char == '\n'
                        || last_char == '\t'
                        || is_unicode_punctuation(last_char)
                    {
                        true
                    } else {
                        false
                    }
                }
            }
            _ => true,
        }
    };
    let is_right_flanking_delimiter_run = {
        match last_char {
            Some(' ') | Some('\n') | Some('\t') | None => false,
            Some(c) if is_unicode_punctuation(c) => {
                let splitted = buffer.split_once(' ');
                if splitted.is_some() {
                    true
                } else {
                    if next_char == None
                        || next_char == Some(&' ')
                        || next_char == Some(&'\n')
                        || next_char == Some(&'\t')
                        || is_unicode_punctuation(*next_char.unwrap())
                    {
                        true
                    } else {
                        false
                    }
                }
            }
            _ => true,
        }
    };

    let mut tmp_tokens = vec![];
    match (
        is_left_flanking_delimiter_run,
        is_right_flanking_delimiter_run,
    ) {
        (true, true) => {
            let splitted_l = buffer.split_once(' ');
            let splitted_r = buffer.rsplit_once(' ');

            if splitted_l.is_none() {
                if buffer.starts_with("_") {
                    match (last_char, next_char) {
                        (Some(lc), Some(&nc)) => {
                            match (is_unicode_punctuation(lc), is_unicode_punctuation(nc)) {
                                (false, false) => prev_buffer.push_str(&buffer),
                                (l, n) => tmp_tokens.push(Token {
                                    token_type: TokenType::Emphasis(match (l, n) {
                                        (true, true) => DelimiterType::Both,
                                        (true, false) => DelimiterType::LeftFlanking,
                                        (false, true) => DelimiterType::RightFlanking,
                                        (false, false) => unreachable!(),
                                    }),
                                    raw: buffer.to_string(),
                                }),
                            }
                        }
                        _ => prev_buffer.push_str(&buffer),
                    }
                } else {
                    tmp_tokens.push(Token {
                        token_type: TokenType::Emphasis(DelimiterType::Both),
                        raw: buffer.to_string(),
                    });
                }
            } else {
                let (head, _) = splitted_l.unwrap();
                let (mid, tail) = splitted_r.unwrap();
                let mid = format!("{} ", mid[head.len()..].to_string());

                tmp_tokens.extend(vec![
                    Token {
                        token_type: TokenType::Emphasis(DelimiterType::LeftFlanking),
                        raw: head.to_string(),
                    },
                    Token {
                        token_type: TokenType::Text,
                        raw: mid,
                    },
                    Token {
                        token_type: TokenType::Emphasis(DelimiterType::RightFlanking),
                        raw: tail.to_string(),
                    },
                ])
            }
        }
        (true, false) => {
            let splitted = buffer.split_once(' ');
            if splitted.is_none() {
                tmp_tokens.push(Token {
                    token_type: TokenType::Emphasis(DelimiterType::LeftFlanking),
                    raw: buffer.to_string(),
                });
            } else {
                let (head, tail) = splitted.unwrap();
                let tail = format!(" {}", tail);

                tmp_tokens.extend(vec![
                    Token {
                        token_type: TokenType::Emphasis(DelimiterType::LeftFlanking),
                        raw: head.to_string(),
                    },
                    Token {
                        token_type: TokenType::Text,
                        raw: tail,
                    },
                ])
            }
        }
        (false, true) => {
            let splitted = buffer.rsplit_once(' ');
            if splitted.is_none() {
                tmp_tokens.push(Token {
                    token_type: TokenType::Emphasis(DelimiterType::RightFlanking),
                    raw: buffer.to_string(),
                });
            } else {
                let (head, tail) = splitted.unwrap();
                let head = format!("{} ", head);

                tmp_tokens.extend(vec![
                    Token {
                        token_type: TokenType::Text,
                        raw: head,
                    },
                    Token {
                        token_type: TokenType::Emphasis(DelimiterType::RightFlanking),
                        raw: tail.to_string(),
                    },
                ])
            }
        }
        (false, false) => prev_buffer.push_str(&buffer),
    }

    if (is_left_flanking_delimiter_run || is_right_flanking_delimiter_run)
        && !prev_buffer.is_empty()
        && !tmp_tokens.is_empty()
    {
        tokens.push(Token {
            token_type: TokenType::Text,
            raw: prev_buffer.clone(),
        });
        prev_buffer.clear();
    }
    tokens.extend(tmp_tokens);
}

#[cfg(test)]
mod tests {
    use crate::token::{DelimiterType, Token, TokenType};

    use super::tokenize_emphasis;

    #[test]
    fn test_emphasis_tokenize() {
        let mut tokens = vec![];
        let mut prev_buffer = "aaa".to_string();
        let mut buffer = "*".to_string();

        tokenize_emphasis(&mut tokens, Some(&'a'), &mut prev_buffer, &mut buffer);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Text,
                    raw: "aaa".to_string(),
                },
                Token {
                    token_type: TokenType::Emphasis(DelimiterType::Both),
                    raw: "*".to_string(),
                },
            ]
        );

        let mut tokens = vec![];
        let mut prev_buffer = "aaa".to_string();
        let mut buffer = "_".to_string();

        tokenize_emphasis(&mut tokens, Some(&'a'), &mut prev_buffer, &mut buffer);
        assert_eq!(tokens, vec![]);
        assert_eq!(prev_buffer, "aaa_");
    }
}
