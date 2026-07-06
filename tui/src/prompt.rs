use std::borrow::Cow;

use chrono::Local;
use reedline::{PromptEditMode, PromptHistorySearchStatus, PromptViMode};

use crate::config::skin::Skin;

pub struct Prompt<'s>(pub &'s Skin);

/// Implementing functions on the type instead of the trait so they can return
/// strings with an extended lifetime
///
/// [`reedline::Prompt`] allow borrowing from the prompt, but we actually do not
/// borrow anything.
impl<'s> Prompt<'s> {
    pub fn render_prompt_left(&self) -> Cow<'static, str> {
        (if self.0.graphical { "🎲" } else { ">>" }).into()
    }

    pub fn render_prompt_right(&self) -> Cow<'static, str> {
        let now = Local::now();
        format!("{:>}", now.format("%m/%d/%Y %I:%M:%S %p")).into()
    }

    pub fn render_prompt_indicator(&self, prompt_mode: PromptEditMode) -> Cow<'static, str> {
        let normal_prompt = if self.0.graphical { "〉" } else { "> " };
        match prompt_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => normal_prompt.into(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal => normal_prompt.into(),
                PromptViMode::Insert => ": ".into(),
            },
            PromptEditMode::Custom(str) => format!("({str})").into(),
        }
    }

    pub fn render_prompt_multiline_indicator(&self) -> Cow<'static, str> {
        "::: ".into()
    }

    pub fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> Cow<'static, str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }
}

impl dices_print_tui::PromptDisplay for Prompt<'_> {
    fn prompt_left(&self) -> Cow<'static, str> {
        Self::render_prompt_left(self)
    }

    fn prompt_indicator(&self) -> Cow<'static, str> {
        Self::render_prompt_indicator(self, PromptEditMode::Default)
    }

    fn prompt_multiline_indicator(&self) -> Cow<'static, str> {
        Self::render_prompt_multiline_indicator(self)
    }
}

impl reedline::Prompt for Prompt<'_> {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Self::render_prompt_left(&self)
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Self::render_prompt_right(&self)
    }

    fn render_prompt_indicator(&self, prompt_mode: PromptEditMode) -> Cow<'_, str> {
        Self::render_prompt_indicator(&self, prompt_mode)
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Self::render_prompt_multiline_indicator(&self)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> Cow<'_, str> {
        Self::render_prompt_history_search_indicator(&self, history_search)
    }

    fn get_prompt_color(&self) -> reedline::Color {
        self.0.theme.prompt()
    }

    fn get_prompt_multiline_color(&self) -> nu_ansi_term::Color {
        self.0.theme.prompt_multiline()
    }

    fn get_indicator_color(&self) -> reedline::Color {
        self.0.theme.prompt_indicator()
    }

    fn get_prompt_right_color(&self) -> reedline::Color {
        self.0.theme.prompt_right()
    }

    fn right_prompt_on_last_line(&self) -> bool {
        false
    }
}
