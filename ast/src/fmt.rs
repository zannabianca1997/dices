//! Common functions to help with formatting stuff

use std::fmt::{Formatter, Write};

use crate::ident::IdentStr;

/// Format a string escaping special chars
pub fn escaped(mut s: &str, f: &mut Formatter<'_>) -> std::fmt::Result {
    while let Some((pos, ch)) = s
        .char_indices()
        .find(|(_, ch)| ['\"', '\\'].contains(ch) || !(ch.is_ascii_graphic() || *ch == ' '))
    {
        f.write_str(&s[..pos])?;
        s = &s[pos + ch.len_utf8()..];

        match ch {
            '\\' => f.write_str(r"\\"),
            '\n' => f.write_str(r"\n"),
            '\r' => f.write_str(r"\r"),
            '\t' => f.write_str(r"\t"),
            '\0' => f.write_str(r"\0"),
            '\'' => f.write_str(r"\'"),
            '\"' => f.write_str(r#"\""#),

            '\x00'..='\x7F' => {
                write!(f, r"\x{:02x}", ch as u8)
            }

            _ => write!(f, r"\u{{{:x}}}", ch as u32),
        }?;
    }
    f.write_str(s)?;
    Ok(())
}

/// Format a string by quoting and escaping it
pub fn quoted(s: &str, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_char('"')?;
    escaped(s, f)?;
    f.write_char('"')?;
    Ok(())
}

/// Quote a string if it's not an identifier, otherwise quote it
pub fn quoted_if_not_ident(s: &str, f: &mut Formatter<'_>) -> std::fmt::Result {
    if let Some(ident) = IdentStr::new(s) {
        write!(f, "{ident}")
    } else {
        quoted(s, f)
    }
}

#[cfg(feature = "pretty")]
#[derive(Clone, Copy)]
/// Structure that prettify in a comma followed by an optional line
pub(crate) struct CommaLine;

#[cfg(feature = "pretty")]
impl<'a, D, A> pretty::Pretty<'a, D, A> for CommaLine
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        allocator.text(",").append(allocator.line())
    }
}
