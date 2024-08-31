use std::io;

use chrono::Local;
use clap::{Parser, ValueEnum};
use derive_more::derive::{Debug, Display, Error, From};
use reedline::{
    DefaultPrompt, Prompt, PromptEditMode, PromptHistorySearchStatus, PromptViMode, Reedline,
    Signal,
};
use termimad::{Alignment, MadSkin};

#[derive(Debug, Clone, Parser)]
#[command(name="dices", version, about, long_about = None)]
pub struct ReplCli {
    #[clap(long, short, default_value_t)]
    /// The grafic level of the REPL
    graphic: Graphic,

    /// If the terminal is light or dark
    #[clap(long, short)]
    teminal: Option<TerminalLightness>,
}

#[derive(Debug, Clone, Copy, Display, ValueEnum)]
pub enum TerminalLightness {
    #[display("light")]
    Light,
    #[display("dark")]
    Dark,
}
#[derive(Debug, Clone, Copy, Display, ValueEnum)]
pub enum Graphic {
    #[display("none")]
    None,
    #[display("ascii")]
    Ascii,
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
    fn banner(&self) -> &str {
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
    fn bye(&self) -> &str {
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
        skin.headers[0].align = Alignment::Right;
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
    #[display("An error happende during io")]
    IO(io::Error),
    #[display("Interrupted.")]
    Interrupted,
}

/// Run the REPL
pub fn repl(args: ReplCli) -> Result<(), ReplFatalError> {
    if atty::is(atty::Stream::Stdin) {
        interactive_repl(args)
    } else {
        detached_repl(args)
    }
}

/// Run the REPL in interactive mode
pub fn interactive_repl(ReplCli { graphic, teminal }: ReplCli) -> Result<(), ReplFatalError> {
    let skin = graphic.skin(teminal);

    skin.print_text(graphic.banner());
    let mut line_editor = Reedline::create();

    loop {
        let sig = line_editor.read_line(&ReplPrompt { graphic })?;
        match sig {
            Signal::Success(buffer) => {
                println!("We processed: {}", buffer);
            }
            Signal::CtrlD => {
                skin.print_text(graphic.bye());
                break Ok(());
            }
            Signal::CtrlC => break Err(ReplFatalError::Interrupted),
        }
    }
}

/// Run the REPL in detached mode (input from a stream)
pub fn detached_repl(ReplCli { graphic, teminal }: ReplCli) -> Result<(), ReplFatalError> {
    todo!()
}
