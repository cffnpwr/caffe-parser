use std::{iter::Peekable, str::Chars};

use crate::token::{Token, TokenType};

pub(super) fn tokenize_list_item_type(chars: &mut Peekable<Chars>, marker: String) -> Token {
    let maybe_checkbox = chars.clone().take(4).collect::<String>();
    if maybe_checkbox != "[ ] " && maybe_checkbox != "[x] " {
        let marker_char = marker.trim().chars().next().unwrap();
        return match marker_char {
            '*' | '+' | '-' => Token {
                token_type: TokenType::BulletListItem,
                raw: marker,
            },
            '0'..='9' => Token {
                token_type: TokenType::OrderedListItem,
                raw: marker,
            },
            _ => unreachable!(),
        };
    }

    let checked = match maybe_checkbox.as_str() {
        "[ ] " => false,
        "[x] " => true,
        _ => unreachable!(),
    };
    for _ in 0..4 {
        chars.next();
    }

    Token {
        token_type: TokenType::CheckListItem(checked),
        raw: format!("{}{}", marker, maybe_checkbox),
    }
}
