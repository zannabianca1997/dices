use std::borrow::Cow;

use chrono::Local;
use reedline::{PromptEditMode, PromptHistorySearchStatus, PromptViMode};

use crate::config::skin::Skin;

pub struct Prompt<'s>(pub &'s Skin);

impl reedline::Prompt for Prompt<'_> {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        (if self.0.emoji { "🎲" } else { "" }).into()
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        let now = Local::now();
        format!("{:>}", now.format("%m/%d/%Y %I:%M:%S %p")).into()
    }

    fn render_prompt_indicator(&self, prompt_mode: PromptEditMode) -> Cow<'_, str> {
        match prompt_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => "〉".into(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal => "〉".into(),
                PromptViMode::Insert => ": ".into(),
            },
            PromptEditMode::Custom(str) => format!("({str})").into(),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        "::: ".into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> Cow<'_, str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }

    fn get_prompt_color(&self) -> reedline::Color {
        self.0.elements.prompts.prompt
    }

    fn get_prompt_multiline_color(&self) -> nu_ansi_term::Color {
        self.0.elements.prompts.multiline
    }

    fn get_indicator_color(&self) -> reedline::Color {
        self.0.elements.prompts.indicator
    }

    fn get_prompt_right_color(&self) -> reedline::Color {
        self.0.elements.prompts.right
    }

    fn right_prompt_on_last_line(&self) -> bool {
        false
    }
}
