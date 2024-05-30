//! A REPL connected to a `dice` engine
#![feature(error_reporter)]
#![feature(iter_intersperse)]
#![feature(box_patterns)]

use std::{
    borrow::Cow,
    error::Report,
    io::{stdout, IsTerminal},
    path::PathBuf,
};

use clap::{Args, Parser, ValueEnum};
use engine::{
    pretty::{Arena, DocAllocator, Pretty},
    EngineBuilder, EvalResult, ParseEvalError, Value,
};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use rand::{rngs::SmallRng, SeedableRng};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct CLI {
    #[clap(short, long)]
    /// The setup file for the REPL
    setup_file: Option<PathBuf>,

    #[clap(flatten)]
    setup: Setup,

    #[clap(short, long)]
    /// Do not close after command execution
    interactive: bool,

    #[clap(
        short,
        long,
        num_args = ..,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    /// Command to run. If missing, an interactive prompt is open
    run: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Args)]
struct Setup {
    #[clap(short, long, default_value = "auto")]
    /// Graphic level of the repl
    graphic: Graphic,

    #[clap(short, long, short)]
    /// Customized prompt for the REPL
    prompt: Option<String>,

    #[clap(short, long, default_value_t = 180)]
    /// Line width before wrapping long values
    width: usize,
}
impl Default for Setup {
    fn default() -> Self {
        Self {
            graphic: Default::default(),
            prompt: None,
            width: 180,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Deserialize, Serialize, Default)]
/// Graphic level of the repl
enum Graphic {
    /// No graphic at all
    None,
    /// Barebone ascii graphic
    Simple,
    /// Emoji Galore
    Nice,
    /// Select None if not a tty or not interactive, else Nice
    #[default]
    Auto,
}
impl Graphic {
    fn header(&self) -> Option<&'static str> {
        match self {
            Graphic::None => None,
            Graphic::Simple => Some(concat!("Dices ", env!("CARGO_PKG_VERSION"))),
            Graphic::Nice => Some(concat!(
                "⛓️ ~ Welcome to DICES ",
                env!("CARGO_PKG_VERSION"),
                " ~ 🐉\n    by zannabianca1997🐺\n"
            )),
            Graphic::Auto => unreachable!(),
        }
    }
    fn prompt(&self) -> &'static str {
        match self {
            Graphic::None => "",
            Graphic::Simple => ">> ",
            Graphic::Nice => "🎲>> ",
            Graphic::Auto => unreachable!(),
        }
    }
    fn bye(&self) -> Option<&'static str> {
        match self {
            Graphic::None => None,
            Graphic::Simple => Some("Bye!"),
            Graphic::Nice => Some("\n⛓️ ~ Thank you for playing! ~ 🐉"),
            Graphic::Auto => unreachable!(),
        }
    }

    fn detect(interactive: bool) -> Self {
        if stdout().is_terminal() && interactive {
            Self::Nice
        } else {
            Self::None
        }
    }
}

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    ReadlineError(#[from] rustyline::error::ReadlineError),
    #[error(transparent)]
    Figment(#[from] figment::Error),
}

fn main() -> Result<(), Error> {
    let CLI {
        setup_file,
        setup,
        interactive,
        run,
    } = CLI::parse();
    // merging setups
    let Setup {
        graphic,
        prompt,
        width,
    } = {
        let mut fig = Figment::new().merge(Serialized::defaults(Setup::default()));
        if let Some(setup_file) = setup_file {
            fig = fig.merge(Toml::file_exact(setup_file))
        } else {
            fig = fig.merge(Toml::file("./dices.toml"))
        }
        fig.merge(Env::prefixed("DICES_"))
            .merge(Serialized::defaults(setup))
            .extract()?
    };
    // choosing defaults
    let interactive = interactive || run.is_none(); // if no command is given, force interaction
    let graphic = if let Graphic::Auto = graphic {
        Graphic::detect(interactive) // choosing a sensible graphic
    } else {
        graphic
    };
    // choosing the default prompt if no prompt was given
    let prompt = prompt
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed(graphic.prompt()));
    // making the eventual command a string
    let run: Option<String> = run.map(|args| args.iter().map(|x| &**x).intersperse(" ").collect());

    let mut engine = EngineBuilder::<SmallRng>::new()
        .with_prelude()
        .rng(SmallRng::from_entropy())
        .build();

    if let Some(header) = graphic.header() {
        println!("{header}")
    }
    if let Some(run) = run.as_ref() {
        // running preliminary command
        if interactive {
            println!("{prompt}{run}");
        }
        // Eval
        let res = engine.eval_line(&run);
        // Print
        print(res, width);
    }
    if interactive {
        let mut rl = rustyline::DefaultEditor::new()?;
        // putting first command into history
        if let Some(run) = run {
            rl.add_history_entry(run)?;
        }
        'repl: loop {
            // Read
            let line = match rl.readline(&prompt) {
                Ok(line) => line,
                // iterrupted is not a error!
                Err(rustyline::error::ReadlineError::Interrupted) => break 'repl,
                Err(err) => Err(err)?,
            };
            // Eval
            let res = engine.eval_line(&line);
            let quitting = res.as_ref().is_ok_and(|v| v.is_quitted());
            // Print
            print(res, width);
            // Loop
            rl.add_history_entry(line)?;
            if quitting {
                break 'repl;
            }
        }
    }
    if let Some(bye) = graphic.bye() {
        println!("{bye}")
    }
    Ok(())
}

fn print(res: Result<EvalResult, ParseEvalError>, width: usize) {
    match res {
        Ok(EvalResult::Ok(Value::Null)) => (),
        Ok(EvalResult::Ok(val)) => {
            // we sadly have to allocate a new arena for every value we print, as there is no way of guarantee that
            // the arena is empty after the printing
            let docs_arena = Arena::<()>::new();
            // now we render the result
            let doc = &*val.pretty(&docs_arena).append(docs_arena.hardline());
            print!("{}", doc.pretty(width))
        }
        Ok(EvalResult::Quitted(params)) => {
            if !params.is_empty() {
                let docs_arena = Arena::<()>::new();
                let val = match Box::<[Value; 1]>::try_from(params) {
                    Ok(box [val]) => val,
                    Err(params) => Value::List(params.into_vec()),
                };
                let doc = &*val.pretty(&docs_arena).append(docs_arena.hardline());
                print!("{}", doc.pretty(width))
            }
        }
        Err(err) => eprintln!("{}", Report::new(err).pretty(true)),
    }
}
