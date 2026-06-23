use const_format::formatcp;

pub const OPENING: &str = formatcp!(
    r#"# Welcome to `dices {}`

Use `help()` for the manual, and `Ctrl+D` to exit."#,
    env!("CARGO_PKG_VERSION")
);

pub const CLOSING: &str = formatcp!(r#"See you at the next game!"#);
