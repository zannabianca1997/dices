use std::{
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    io::{self, stderr, stdin, stdout, Write as _},
    mem,
    path::PathBuf,
    rc::Rc,
};

use chrono::Local;
pub use clap::Parser as ClapParser;
use clap::ValueEnum;
use derive_more::derive::{Debug, Display, Error, From};
use dices_ast::value::{Value, ValueNull};
use dices_engine::Engine;
use intrisics::{Quitted, REPLIntrisics};
use pretty::Pretty;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use reedline::{Prompt, PromptEditMode, PromptHistorySearchStatus, PromptViMode, Reedline, Signal};
use serde::{Deserialize, Serialize};
use setup::Setup;
use termimad::{
    minimad::{Compound, Line, Text},
    terminal_size, Alignment, FmtText, MadSkin,
};
use typed_arena::Arena;

mod intrisics;
mod setup;

#[derive(Debug, Clone, ClapParser)]
#[command(name="dices", version, about, long_about = None)]
pub struct ReplCli {
    /// File for the default options for the REPL
    #[clap(long = "setup", short = 'S')]
    file_setup: Option<PathBuf>,

    #[clap(flatten)]
    cli_setup: setup::Setup,

    /// If `run` is given, do not close after command execution.
    #[clap(long, short)]
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

#[derive(Debug, Clone, Copy, Display, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalLightness {
    #[display("light")]
    Light,
    #[display("dark")]
    Dark,
}

#[derive(Debug, Clone, Copy, Display, ValueEnum, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Graphic {
    /// No graphic
    #[display("none")]
    None,
    /// Only ascii graphic
    #[display("ascii")]
    Ascii,
    /// Fancy graphic, with emojis
    #[display("fancy")]
    Fancy,
}
impl Default for Graphic {
    fn default() -> Self {
        if atty::is(atty::Stream::Stdout) {
            Graphic::Fancy
        } else {
            Graphic::None
        }
    }
}
impl Graphic {
    const fn banner(&self) -> &str {
        match self {
            Graphic::None => "",
            Graphic::Ascii => concat!(
                "Welcome to dices ",
                env!("CARGO_PKG_VERSION"),
                "\n\nUse help() for the manual, and quit() or Ctrl+D to exit."
            ),
            Graphic::Fancy => concat!(
                "⛓️🐉 ~ ***Welcome to dices ",
                env!("CARGO_PKG_VERSION"),
                "*** ~ ⛓️🐉\n\nUse `help()` for the manual, and `quit()` or `Ctrl+D` to exit."
            ),
        }
    }
    const fn prompt(&self) -> &str {
        match self {
            Graphic::None => "",
            Graphic::Ascii => ">>> ",
            Graphic::Fancy => "🎲> ",
        }
    }
    const fn prompt_cont(&self) -> &str {
        match self {
            Graphic::None => "",
            Graphic::Ascii => "... ",
            Graphic::Fancy => "🎲. ",
        }
    }
    const fn bye(&self) -> &str {
        match self {
            Graphic::None => "",
            Graphic::Ascii => "\nSee you at the next game!",
            Graphic::Fancy => "\n⛓️🐉 ~ *See you at the next game!* ~ ⛓️🐉",
        }
    }

    fn skin(&self, light: Option<TerminalLightness>) -> termimad::MadSkin {
        let mut skin = match self {
            Graphic::None | Graphic::Ascii => termimad::MadSkin::no_style(),
            Graphic::Fancy => match light {
                Some(TerminalLightness::Light) => MadSkin::default_light(),
                Some(TerminalLightness::Dark) => MadSkin::default_dark(),
                None => MadSkin::default(),
            },
        };
        // Disabling centered text, I find it confusing
        skin.headers[0].align = Alignment::Left;
        skin
    }
}

pub struct ReplPrompt {
    graphic: Graphic,
}
impl Prompt for ReplPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        match self.graphic {
            Graphic::None => "",
            Graphic::Ascii => ">>",
            Graphic::Fancy => "🎲",
        }
        .into()
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        let now = Local::now();
        format!("{:>}", now.format("%m/%d/%Y %I:%M:%S %p")).into()
    }

    fn render_prompt_indicator(
        &self,
        prompt_mode: reedline::PromptEditMode,
    ) -> std::borrow::Cow<str> {
        match prompt_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => "> ".into(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal => "> ".into(),
                PromptViMode::Insert => ": ".into(),
            },
            PromptEditMode::Custom(str) => format!("{str} ").into(),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        match self.graphic {
            Graphic::None => "",
            Graphic::Ascii => "... ",
            Graphic::Fancy => "🎲. ",
        }
        .into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        // Copying reedline DefaultPrompt
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        format!("({}reverse-search: {}) ", prefix, history_search.term).into()
    }
}

#[derive(Debug, Display, Error, From)]
pub enum ReplFatalError {
    #[display("Error during IO")]
    IO(io::Error),
    #[display("Error during execution")]
    Run(dices_engine::EvalStrError<REPLIntrisics>),
    #[display("Error during extraction of the setup")]
    Setup(figment::Error),
    #[display("Interrupted.")]
    Interrupted,
}

/// Run the REPL
pub fn repl(
    ReplCli {
        file_setup,
        cli_setup,
        interactive,
        run,
    }: &ReplCli,
) -> Result<(), ReplFatalError> {
    let (
        Setup {
            graphic, terminal, ..
        },
        err,
    ) = match setup::Setup::extract_setups(file_setup.as_ref(), cli_setup).map(|setup| {
        let repl_result = repl_with_setup(interactive, run, &setup);
        (setup, repl_result)
    }) {
        Ok((_, Ok(()))) => return Ok(()),
        Ok((setup, Err(err))) => (setup, err),
        Err(err) => (Setup::default(), err.into()),
    };
    let graphic = graphic.unwrap_or_default();
    let skin = graphic.skin(terminal);

    print_err::<true>(graphic, &skin, &err).unwrap();

    Err(err)
}

fn repl_with_setup(
    interactive: &bool,
    run: &Option<Vec<String>>,
    setup::Setup {
        graphic,
        terminal,
        seed,
    }: &Setup,
) -> Result<(), ReplFatalError> {
    // Identify the default graphic if not given
    let graphic = graphic.unwrap_or_default();

    // Boxing the graphic
    let graphic = Rc::new(graphic);
    // Creating the skin
    let skin = Rc::new(graphic.skin(*terminal));
    // Initializing the engine
    let engine_builder = dices_engine::EngineBuilder::new()
        .inject_intrisics_data(intrisics::Data::new(graphic.clone(), skin.clone()));
    let engine_builder = if let Some(seed) = seed {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);

        engine_builder.with_rng(Xoshiro256PlusPlus::seed_from_u64(hasher.finish()))
    } else {
        engine_builder.with_rng_from_entropy()
    };
    let mut engine: dices_engine::Engine<Xoshiro256PlusPlus, REPLIntrisics, _> =
        engine_builder.build();

    if let Some(run) = run {
        // joining of the shell arguments
        let cmd = run.join(" ");
        // running in the new engine
        let value = engine.eval_str(&cmd)?;
        // printing the result of the init command
        print_value(
            *graphic,
            &skin,
            &value,
            *interactive, // skip printing `null` if the console is interactive
        )?;
        if !(*interactive && value.is_null()) {
            println!();
        }

        if !interactive {
            // runned the single command, exiting.
            return Ok(());
        }
    }

    // Printing the initial banner
    skin.print_text(graphic.banner());

    if atty::is(atty::Stream::Stdin) {
        interactive_repl(graphic.clone(), skin.clone(), &mut engine)?;
    } else {
        detached_repl(graphic.clone(), skin.clone(), &mut engine)?;
    };

    // Print the out banner
    skin.print_text(graphic.bye());

    Ok(())
}

/// Run the REPL in interactive mode
pub fn interactive_repl(
    graphic: Rc<Graphic>,
    skin: Rc<MadSkin>,
    engine: &mut Engine<Xoshiro256PlusPlus, REPLIntrisics, intrisics::Data>,
) -> Result<(), ReplFatalError> {
    // Creating the editor
    let mut line_editor = Reedline::create();
    // REPL loop
    loop {
        let sig = line_editor.read_line(&ReplPrompt { graphic: *graphic })?;
        match sig {
            Signal::Success(line) => match engine.eval_str(&line) {
                Ok(value) => print_value(*graphic, &skin, &value, true)?,
                Err(err) => {
                    // need to catch the quitting error
                    match engine.injected_intrisics_data_mut().quitted_mut().take() {
                        Quitted::Yes(value) => {
                            let _ = err;
                            print_value(*graphic, &skin, &value, true)?;
                            break;
                        }
                        Quitted::No => (),
                        Quitted::Fatal(repl_fatal_error) => return Err(repl_fatal_error),
                    }
                    print_err::<false>(*graphic, &skin, err)?;
                }
            },
            Signal::CtrlD => {
                break;
            }
            Signal::CtrlC => return Err(ReplFatalError::Interrupted),
        }
    }
    Ok(())
}

/// Run the REPL in detached mode (input from a stream)
pub fn detached_repl(
    graphic: Rc<Graphic>,
    skin: Rc<MadSkin>,
    engine: &mut Engine<Xoshiro256PlusPlus, REPLIntrisics, intrisics::Data>,
) -> Result<(), ReplFatalError> {
    // REPL loop
    for line in stdin().lines() {
        let line = line?;
        println!("{}{}", graphic.prompt(), line);
        match engine.eval_str(&line) {
            Ok(value) => print_value(*graphic, &skin, &value, true)?,
            Err(err) => {
                // need to catch the quitting error
                match engine.injected_intrisics_data_mut().quitted_mut().take() {
                    Quitted::Yes(value) => {
                        let _ = err;
                        print_value(*graphic, &skin, &value, true)?;
                        break;
                    }
                    Quitted::No => (),
                    Quitted::Fatal(repl_fatal_error) => return Err(repl_fatal_error),
                }
                print_err::<false>(*graphic, &skin, err)?;
            }
        };
    }
    Ok(())
}

/// Print a value
fn print_value(
    graphic: Graphic,
    _skin: &MadSkin,
    value: &Value<REPLIntrisics>,
    skip_nulls: bool,
) -> Result<(), ReplFatalError> {
    if skip_nulls && value == &Value::Null(ValueNull) {
        // do not print null values
        return Ok(());
    }
    if graphic == Graphic::None {
        println!("{}", value);
        return Ok(());
    }
    let arena = pretty::Arena::<()>::new();
    value
        .pretty(&arena)
        .render(terminal_size().0 as _, &mut stdout().lock())?;
    Ok(())
}

/// Print an error
fn print_err<const FATAL: bool>(
    graphic: Graphic,
    skin: &MadSkin,
    error: impl Error,
) -> Result<(), ReplFatalError> {
    let lines = Arena::new();
    let mut text = Text { lines: vec![] };

    let mut compounds = vec![];
    if graphic == Graphic::Fancy {
        compounds.push(Compound::raw_str(if FATAL { "🛑 " } else { "🚨 " }).bold());
    }
    compounds.push(Compound::raw_str("Error: ").bold());
    compounds.append(
        &mut termimad::minimad::parse_inline(lines.alloc_str(&error.to_string())).compounds,
    );
    text.lines
        .push(Line::new_paragraph(mem::take(&mut compounds)));

    if let Some(source) = error.source() {
        let mut source = Some(source);

        text.lines.push(Line::raw_str(""));
        text.lines.push(Line::new_paragraph(vec![
            Compound::raw_str("Caused by:").italic()
        ]));

        while let Some(current) = source {
            source = current.source();

            text.lines.push(Line::new_list_item(
                1,
                termimad::minimad::parse_inline(lines.alloc_str(&current.to_string())).compounds,
            ));
        }
    }

    writeln!(
        stderr().lock(),
        "{}",
        FmtText::from_text(skin, text, Some(terminal_size().0 as _))
    )?;

    Ok(())
}
