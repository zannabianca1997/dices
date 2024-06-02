//! Collection of manual pages

use std::{
    borrow::Cow,
    fmt::{Display, Write as _},
};

use lazy_regex::{regex, Lazy, Regex};

static INDEX: phf::Map<&'static str, Page> = include!(concat!(env!("OUT_DIR"), "/index.rs"));

#[derive(Debug, Clone)]
pub struct Page {
    pub title: &'static str,
    pub content: Cow<'static, str>,
}
impl Page {
    /// Substitute manual links `[man](man://...)`
    pub fn fix_man_links<D: AsRef<str>>(&mut self, fixer: impl Fn(&str, &str) -> D) {
        static LINKS_RE: &Lazy<Regex> = regex!(r"\[([^\]]+)\]\(man://((?:\w+/)*\w+)\)");
        if LINKS_RE.is_match(&self.content) {
            self.content = Cow::Owned(
                LINKS_RE
                    .replace_all(&self.content, |capture: &regex::Captures| {
                        fixer(
                            capture.get(1).unwrap().as_str(),
                            capture.get(2).unwrap().as_str(),
                        )
                    })
                    .into_owned(),
            );
        }
    }

    /// Substitute the code example with the results from the given doc runner
    pub fn run_docs<R: DocRunner>(&mut self, runner: impl Fn() -> R) {
        static DOC_RE: &Lazy<Regex> = regex!(r"^```\s*dices\s*\n(.*(?:\n.*)*?)\n```\s*$"m);
        if DOC_RE.is_match(&self.content) {
            // replace the docs
            self.content = Cow::Owned(
                DOC_RE
                    .replace_all(&self.content, |capture: &regex::Captures| {
                        let commands = capture
                            .get(1)
                            .unwrap()
                            .as_str()
                            .lines()
                            .filter_map(|l| l.strip_prefix(">> "));
                        let mut runner = runner();
                        let mut buf = String::new();
                        writeln!(&mut buf, "```dices,runned").unwrap();
                        for cmd in commands {
                            writeln!(&mut buf, "{}{}", runner.prompt(), cmd,).unwrap();
                            writeln!(&mut buf, "{}", runner.exec(cmd)).unwrap();
                        }
                        writeln!(&mut buf, "```").unwrap();
                        buf
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
