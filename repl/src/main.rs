//! A REPL connected to a `dice` engine
#![feature(error_reporter)]

use std::error::Report;

use engine::{
    pretty::{Arena, DocAllocator, Pretty},
    Engine, Value,
};
use rand::rngs::SmallRng;

fn main() -> rustyline::Result<()> {
    let mut rl = rustyline::DefaultEditor::new()?;
    let mut engine = Engine::<SmallRng>::new();
    'repl: loop {
        // Read
        let line = match rl.readline(">> ") {
            Ok(line) => line,
            // iterrupted is not a error!
            Err(rustyline::error::ReadlineError::Interrupted) => break 'repl,
            Err(err) => return Err(err),
        };
        // Eval
        let res = engine.eval_line(&line);
        // Print
        match res {
            Ok(Value::Null) => (),
            Ok(val) => {
                // we sadly have to allocate a new arena for every value we print, as there is no way of guarantee that
                // the arena is empty after the printing
                let docs_arena = Arena::<()>::new();
                // now we render the result
                let doc = &*val.pretty(&docs_arena).append(docs_arena.hardline());
                print!("{}", doc.pretty(80))
            }
            Err(err) => eprintln!("{}", Report::new(err).pretty(true)),
        }
        // Loop
        rl.add_history_entry(line)?;
    }
    println!("Bye!");
    Ok(())
}
