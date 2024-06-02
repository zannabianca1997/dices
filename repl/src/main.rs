//! A REPL connected to a `dice` engine
#![feature(error_reporter)]
#![feature(iter_intersperse)]
#![feature(box_patterns)]

use std::{
    borrow::Cow,
    error::Report,
    fmt::Write as _,
    io::{stdout, IsTerminal},
    mem,
    path::PathBuf,
};

use clap::{Args, Parser, ValueEnum};
use engine::{
    pretty::{Arena, DocAllocator, Pretty},
    Callbacks, Engine, EngineBuilder, EvalResult, ParseEvalError, Value,
};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use man::DocRunner;
use rand::{rngs::SmallRng, SeedableRng};
use serde::{Deserialize, Serialize};
use termimad::MadSkin;
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
    #[clap(short, long)]
    /// Graphic level of the repl
    graphic: Option<Graphic>,

    #[clap(short, long, short)]
    /// Customized prompt for the REPL
    prompt: Option<String>,

    #[clap(short, long, default_value_t = 120)]
    /// Line width before wrapping long values
    width: usize,
}
impl Default for Setup {
    fn default() -> Self {
        Self {
            graphic: Default::default(),
            prompt: None,
            width: 120,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Deserialize, Serialize)]
/// Graphic level of the repl
enum Graphic {
    /// No graphic at all
    None,
    /// Barebone ascii graphic
    Simple,
    /// Emoji Galore
    Nice,
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
        }
    }
    fn prompt(&self) -> &'static str {
        match self {
            Graphic::None => "",
            Graphic::Simple => ">> ",
            Graphic::Nice => "🎲>> ",
        }
    }
    fn bye(&self) -> Option<&'static str> {
        match self {
            Graphic::None => None,
            Graphic::Simple => Some("Bye!"),
            Graphic::Nice => Some("\n⛓️ ~ Thank you for playing! ~ 🐉"),
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
    let graphic = graphic.unwrap_or_else(|| Graphic::detect(interactive));
    let skin = match graphic {
        Graphic::None | Graphic::Simple => MadSkin::no_style(),
        Graphic::Nice => MadSkin::default(),
    };
    // choosing the default prompt if no prompt was given
    let prompt = prompt
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed(graphic.prompt()));
    // making the eventual command a string
    let run: Option<String> = run.map(|args| args.iter().map(|x| &**x).intersperse(" ").collect());

    let mut engine = EngineBuilder::new()
        .rng(SmallRng::from_entropy())
        .callbacks(REPLCallbacks {
            prompt: &prompt,
            width: &width,
            skin: &skin,
        })
        .build();

    if let Some(header) = interactive.then(|| graphic.header()).flatten() {
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
        match print(res, width, &skin) {
            Ok(ok) => print!("{ok}"),
            Err(err) => eprint!("{err}"),
        };
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
            match print(res, width, &skin) {
                Ok(ok) => print!("{ok}"),
                Err(err) => eprint!("{err}"),
            };
            // Loop
            rl.add_history_entry(line)?;
            if quitting {
                break 'repl;
            }
        }
    }
    if let Some(bye) = interactive.then(|| graphic.bye()).flatten() {
        println!("{bye}")
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct REPLCallbacks<'s> {
    prompt: &'s str,
    width: &'s usize,
    skin: &'s termimad::MadSkin,
}
impl Callbacks for REPLCallbacks<'_> {
    const PRINT_AVAIL: bool = true;

    fn print(&mut self, value: Value) {
        match print(Ok(EvalResult::Ok(value)), *self.width, self.skin) {
            Ok(ok) => print!("{ok}"),
            Err(err) => eprint!("{err}"),
        }
    }

    const HELP_AVAIL: bool = true;

    fn help(&mut self, mut page: man::Page) {
        page.run_docs(|| REPLDocRunner::new(self.prompt, self.width, self.skin));
        self.skin.print_text(&page.content);
        println!();
    }
}

fn print(
    res: Result<EvalResult, ParseEvalError>,
    width: usize,
    _skin: &MadSkin,
) -> Result<String, String> {
    let mut buf = String::new();
    match res {
        Ok(EvalResult::Ok(Value::Null)) => Ok(buf),
        Ok(EvalResult::Ok(val)) => {
            // we sadly have to allocate a new arena for every value we print, as there is no way of guarantee that
            // the arena is empty after the printing
            let docs_arena = Arena::<()>::new();
            // now we render the result
            let doc = &*val.pretty(&docs_arena).append(docs_arena.hardline());
            write!(&mut buf, "{}", doc.pretty(width)).unwrap();
            Ok(buf)
        }
        Ok(EvalResult::Quitted(params)) => {
            if !params.is_empty() {
                let docs_arena = Arena::<()>::new();
                let val = match Box::<[Value; 1]>::try_from(params) {
                    Ok(box [val]) => val,
                    Err(params) => Value::List(params.into_vec()),
                };
                let doc = &*val.pretty(&docs_arena).append(docs_arena.hardline());
                write!(&mut buf, "{}", doc.pretty(width)).unwrap();
            }
            Ok(buf)
        }
        Err(err) => {
            writeln!(&mut buf, "{}", Report::new(err).pretty(true)).unwrap();
            Err(buf)
        }
    }
}

#[derive(Debug, Clone)]
struct REPLDocRunner<'s> {
    prompt: &'s str,
    width: &'s usize,
    skin: &'s MadSkin,
    engine: Engine<SmallRng, RunnerCallbacks<'s>>,
}
impl<'s> REPLDocRunner<'s> {
    fn new(prompt: &'s str, width: &'s usize, skin: &'s MadSkin) -> Self {
        Self {
            prompt,
            width,
            skin,
            engine: EngineBuilder::new()
                .rng(SmallRng::seed_from_u64(42)) // know seed so the doc are constant
                .callbacks(RunnerCallbacks {
                    width,
                    skin,
                    printed: String::new(),
                })
                .build(),
        }
    }
}
impl DocRunner for REPLDocRunner<'_> {
    fn prompt(&self) -> impl std::fmt::Display {
        self.prompt
    }

    fn exec(&mut self, cmd: &str) -> impl std::fmt::Display {
        let res = self.engine.eval_line(cmd);
        let (Ok(res) | Err(res)) = print(res, *self.width, self.skin);
        let mut printed = mem::take(&mut self.engine.callbacks_mut().printed);
        printed.push_str(&res);
        printed
    }
}

#[derive(Debug, Clone)]
struct RunnerCallbacks<'s> {
    width: &'s usize,
    skin: &'s termimad::MadSkin,

    printed: String,
}
impl Callbacks for RunnerCallbacks<'_> {
    const PRINT_AVAIL: bool = true;

    fn print(&mut self, value: Value) {
        let (Ok(printed) | Err(printed)) =
            print(Ok(EvalResult::Ok(value)), *self.width, &self.skin);
        self.printed.push_str(&printed)
    }

    const HELP_AVAIL: bool = false;

    fn help(&mut self, _page: man::Page) {
        panic!()
    }
}
