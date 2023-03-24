use std::io::{stdin, stdout, Write};

use dices::Cmd;

fn main() {
    let mut buf = String::new();
    print!("Insert a command: ");
    stdout().flush().ok();
    stdin().read_line(&mut buf).expect("Error in reading input");
    match buf.parse::<Cmd>() {
        Ok(cmd) => println!("Parsed: {cmd:#?}"),
        Err(err) => println!("Error: {err}"),
    }
}
