//! Collection of manual pages

use std::{borrow::Cow, fmt::Display};

use lazy_regex::{regex, regex_replace, regex_replace_all, Lazy, Regex};

static INDEX: phf::Map<&'static str, Page> = include!(concat!(env!("OUT_DIR"), "/index.rs"));

#[derive(Debug, Clone)]
pub struct Page {
    pub title: &'static str,
    pub content: Cow<'static, str>,
}
impl Page {
    /// Substitute the code example with the results from the given doc runner
    pub fn run_docs<R: DocRunner>(&mut self, runner: impl Fn() -> R) {
        static DOC_RE: &Lazy<Regex> = regex!(r"^```\s*dices\s*\n((?:.*\n)*?)```\s*$"m);
        if DOC_RE.is_match(&self.content) {
            // replace the docs
            self.content = Cow::Owned(
                DOC_RE
                    .replace_all(&self.content, |capture: &regex::Captures| {
                        format!("```dices\n{}\n```", capture.get(1).unwrap().as_str())
                    })
                    .into_owned(),
            );
        }
    }
}

pub fn man(page: &str) -> Option<Page> {
    INDEX.get(page).cloned()
}

/// A type that can run a doc
pub trait DocRunner {
    /// Get the prompt
    fn prompt(&self) -> impl Display;
    /// Run the next command
    fn exec(&mut self, cmd: &str) -> impl Display;
}
