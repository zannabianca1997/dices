#![feature(error_reporter)]
#![feature(iter_intersperse)]

use std::{
    error::Report,
    io::{self, stderr, stdin, Write},
};

use clap::{Parser, ValueEnum};
use rustyline::{error::ReadlineError, history::MemHistory, Config, Editor};
use termimad::{minimad::TextTemplate, Alignment, FmtText, MadSkin};
use thiserror::Error;

use dices::{Cmd, CmdError, State};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output format and pretty-printing
    #[arg(long, short, value_enum, default_value_t)]
    skin: SkinSetup,
    /// Interactive mode: present a prompt and more readable infos
    #[arg(long, short, default_value_t=atty::is(atty::Stream::Stdout))]
    interactive: bool,
    /// Command to run, and exit after
    #[arg(long, short)]
    command: Option<String>,
}

#[derive(ValueEnum, Clone)]
enum SkinSetup {
    Pretty,
    PrettyDark,
    PrettyLight,
    Plain,
}

impl Default for SkinSetup {
    fn default() -> Self {
        if atty::is(atty::Stream::Stdout) {
            SkinSetup::Pretty
        } else {
            SkinSetup::Plain
        }
    }
}

impl SkinSetup {
    fn skin(&self) -> MadSkin {
        match self {
            SkinSetup::Pretty => {
                let mut s = MadSkin::default();
                s.headers[0].align = Alignment::Left;
                s
            }
            SkinSetup::PrettyDark => {
                let mut s = MadSkin::default_dark();
                s.headers[0].align = Alignment::Left;
                s
            }
            SkinSetup::PrettyLight => {
                let mut s = MadSkin::default_light();
                s.headers[0].align = Alignment::Left;
                s
            }
            SkinSetup::Plain => MadSkin::no_style(),
        }
    }

    /// Returns `true` if the skin setup is pretty
    #[must_use]
    fn is_pretty(&self) -> bool {
        matches!(self, Self::Pretty | Self::PrettyDark | Self::PrettyLight)
    }
}

#[derive(Debug, Error)]
enum MainError {
    #[error(transparent)]
    RustyLine(#[from] ReadlineError),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("Interrupted")]
    Interrupted,
}

fn main() -> Result<(), MainError> {
    let args = Args::parse();

    let mut state = State::new();
    let skin = args.skin.skin();

    if let Some(cmd) = args.command {
        // single command mode
        let res = cmd
            .parse::<Cmd>()
            .map_err(CmdError::Parsing)
            .and_then(|cmd| cmd.execute(&mut state));
        match res {
            Ok(output) => output.print(&skin, args.skin.is_pretty(), args.interactive),
            Err(err) => println!("Error: {}", Report::new(err).pretty(args.interactive)),
        }
        return Ok(());
    }

    let mut rl = if args.interactive {
        Some(Editor::<(), _>::with_history(
            Config::default(),
            MemHistory::new(),
        )?)
    } else {
        None
    };

    if args.interactive {
        let header_template = TextTemplate::from(if args.skin.is_pretty() {
            "# ⛓️  Welcome to ${name} ${version} 🐉\n\nInput `?` to see a list of commands"
        } else {
            "# Welcome to ${name} ${version}\n\nInput `?` to see a list of commands"
        });
        let mut header_expander = header_template.expander();
        header_expander
            .set("version", env!("CARGO_PKG_VERSION"))
            .set("name", env!("CARGO_PKG_NAME"));
        let header = FmtText::from_text(&skin, header_expander.expand(), None);
        print!("{header}");
    }

    loop {
        // Read
        let readline = if let Some(rl) = rl.as_mut() {
            rl.readline(if args.skin.is_pretty() {
                "🎲 >> "
            } else {
                ">> "
            })
        } else {
            // simulate readline, no editing
            let mut buf = String::new();
            let n = stdin().read_line(&mut buf)?;
            if n > 0 {
                Ok(buf)
            } else {
                Err(ReadlineError::Eof)
            }
        };
        // Eval
        let res = match readline {
            Ok(line) => {
                if let Some(rl) = rl.as_mut() {
                    rl.add_history_entry(line.as_str())?;
                }
                line.parse::<Cmd>().map_err(CmdError::Parsing)
            }
            Err(ReadlineError::Interrupted) => {
                return Err(MainError::Interrupted);
            }
            Err(ReadlineError::Eof) => {
                return Ok(());
            }
            Err(err) => {
                return Err(err.into());
            }
        }
        .and_then(|cmd| cmd.execute(&mut state));
        // Print
        match res {
            Ok(output) => {
                output.print(&skin, args.skin.is_pretty(), args.interactive);
                if output.is_final() {
                    return Ok(());
                }
            }
            Err(err) => writeln!(
                stderr(),
                "Error: {}",
                Report::new(err).pretty(args.interactive)
            )
            .unwrap(),
        }
    }
}
