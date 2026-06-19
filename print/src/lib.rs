#![doc = include_str!("../README.md")]

pub mod theme;

#[derive(Debug, Clone, Copy)]
pub enum Annotation {
    /// Graphical fluff
    ///
    /// Annotated text is a graphical fluff. Should be skipped on plain output.
    Fluff,

    /// Prompt
    ///
    /// Annotated text is part of the interactive prompt
    Prompt(Option<PromptElement>),

    /// Markdown
    ///
    /// General styled contend for the banners and the manual
    Markdown(Option<MarkdownElement>),

    /// Value
    ///
    /// Annotated text is part of a dices value representation
    Value(Option<ValueElement>),
}

#[derive(Debug, Clone, Copy)]
pub enum PromptElement {
    /// Indicator shown before the user input (e.g. the chevron)
    Indicator,
    /// Continuation indicator shown on multiline input
    Multiline,
    /// Right-aligned prompt (e.g. the clock)
    Right,
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum MarkdownElement {
    Header { level: u8 },
    InlineCode,
    Bold,
    Italic,
}

#[derive(Debug, Clone, Copy)]
pub enum ValueElement {
    /// Null literal
    Null,
    /// Boolean literal
    Bool { value: bool },
    /// Integer literal
    Integer,
    /// String literal
    String {
        /// Inside an escape code
        escape: bool,
    },
    /// Delimiter (bracket, map angular parentheses, etc)
    Delimiter {
        /// Kind of the delimiter
        kind: DelimiterKind,
        /// Depth of the delimiter nesting
        depth: u8,
    },
    /// Punctuators
    Punctuator,
    /// Injected values
    Injected,
}

#[derive(Debug, Clone, Copy)]
pub enum DelimiterKind {
    List,
    Map,
}
