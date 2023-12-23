use crate::token::Token;

use self::block_tokenizer::block_tokenize;

mod block_tokenizer;
mod inline_tokenizer;

pub(crate) fn tokenize(input: &str) -> Vec<Token> {
    block_tokenize(input)
}
