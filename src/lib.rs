mod ast;
mod token;
mod tokenizer;
mod util;

pub fn main() {
    let input = "# Hello, world!";
    let tokens = tokenizer::tokenize(input);

    print!("{:#?}", tokens)
}
