#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
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

    /// Ast
    ///
    /// Annotated text is part of a dices AST representation
    Ast(Option<AstElement>),

    /// Annotated text is an error message
    Error(Option<ErrorElement>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptElement {
    /// Indicator shown before the user input (e.g. the chevron)
    Indicator,
    /// Continuation indicator shown on multiline input
    Multiline,
    /// Right-aligned prompt (e.g. the clock)
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MarkdownElement {
    Header {
        level: u8,
    },
    Code {
        inline: bool,
    },
    Bold,
    Italic,
    Paragraph,
    List {
        style: ListStyle,
        element: Option<List>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum List {
    /// A list item
    Item,
    /// A list item's marker (the bullet or number)
    Marker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStyle {
    /// Numbered list (`1.`, `2.`, …)
    Ordered,
    /// Bulleted list (`-`)
    Unordered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        nesting: u8,
    },
    /// Punctuators
    Punctuator,
    /// Injected values
    Injected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstElement {
    /// Identifier
    Ident,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimiterKind {
    List,
    Map,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorElement {
    Message,
    Source,
}
