//! Render the manual into a resource blob for the badge app.
//!
//! ```sh
//! cargo run -p dices-badge --bin prerender -- <output path> [width]
//! ```
//!
//! The badge links this blob (see [`dices_badge::manual_resource`]) instead of
//! the markdown renderer, which does not fit its ELF loader's memory budget.

use std::process::ExitCode;

use dices_badge::manual_resource::build_blob;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(output) = args.next() else {
        eprintln!("usage: prerender <output path> [width]");
        return ExitCode::FAILURE;
    };
    let width: usize = match args.next() {
        Some(value) => match value.parse() {
            Ok(width) => width,
            Err(error) => {
                eprintln!("invalid width {value:?}: {error}");
                return ExitCode::FAILURE;
            }
        },
        None => dices_badge::DEFAULT_WIDTH,
    };

    let blob = build_blob(width);
    if let Err(error) = std::fs::write(&output, &blob) {
        eprintln!("could not write {output}: {error}");
        return ExitCode::FAILURE;
    }

    println!("wrote {output} ({} bytes, width {width})", blob.len());
    ExitCode::SUCCESS
}
