use std::io::{read_to_string, stdin};

use dices_ast::parse::parse_file;

fn main() {
    let src = read_to_string(stdin()).expect("Cannot read stdin");
    match parse_file(&src) {
        Ok(exprs) => println!("{exprs:?}"),
        Err(err) => eprintln!("{err}"),
    }
}
