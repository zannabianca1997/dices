//! Prerendered manual, shipped as a loadable resource.
//!
//! Rendering the manual on the badge would mean linking the markdown parser and
//! the whole page renderer into the app; on a device that has to fit the ELF
//! loader's memory budget that is the single biggest avoidable cost. Instead the
//! pages are rendered once, at build time, with the badge's fixed theme and
//! layout ([`build_blob`], run by `src/bin/prerender.rs`), and the app only
//! looks them up.
//!
//! Blob format (little endian):
//!
//! ```text
//! "DMAN" u8[4]
//! u32 page_count
//! page_count × {
//!     u16 component_count
//!     component_count × u16        // the page path, as in `ManPage::path`
//!     u32 text_len
//!     u8[text_len]                 // rendered text, UTF-8
//! }
//! ```

use std::collections::HashMap;

use dices_man::{ManPage, Manual};

const MAGIC: &[u8; 4] = b"DMAN";

/// Render every page of the manual with the badge's layout and serialize it.
///
/// The example code blocks are evaluated with a fixed seed, so the result is
/// deterministic.
pub fn build_blob(width: usize) -> Vec<u8> {
    let manual = Manual::new();
    let root = manual.root();

    let mut pages: Vec<(Vec<u16>, String)> = Vec::new();
    for page in root.descendants() {
        let text = render_page(&page, width);
        pages.push((page.path().to_vec(), text));
    }

    let mut blob = Vec::new();
    blob.extend_from_slice(MAGIC);
    blob.extend_from_slice(&(pages.len() as u32).to_le_bytes());
    for (path, text) in pages {
        blob.extend_from_slice(&(path.len() as u16).to_le_bytes());
        for component in path {
            blob.extend_from_slice(&component.to_le_bytes());
        }
        blob.extend_from_slice(&(text.len() as u32).to_le_bytes());
        blob.extend_from_slice(text.as_bytes());
    }
    blob
}

/// Render one page exactly like the badge does, without color.
fn render_page(page: &ManPage, width: usize) -> String {
    use dices_print::{Pretty as _, manual, markdown::DefaultCodeRender};
    use dices_print_cli::{examples::CliCodeRender, render as render_doc};
    use pretty::Arena;

    let arena = Arena::new();
    let ctx = manual::Ctx::new((CliCodeRender::new(crate::BadgePrompt), DefaultCodeRender));
    let mut buffer = Vec::new();
    render_doc(
        page.with_ctx(ctx),
        &arena,
        &mut buffer,
        width,
        false,
        |_| dices_print::theme::Style::default(),
    )
    .expect("rendering to a memory buffer cannot fail");
    String::from_utf8(buffer).expect("rendered text is valid utf-8")
}

/// A parsed prerendered manual.
#[derive(Debug, Default)]
pub struct PrerenderedManual {
    pages: HashMap<Vec<u16>, String>,
}

impl PrerenderedManual {
    /// Parse a blob produced by [`build_blob`].
    pub fn parse(blob: &[u8]) -> Result<Self, ParseError> {
        let mut reader = Reader { data: blob, pos: 0 };

        if reader.take(4)? != MAGIC {
            return Err(ParseError::BadMagic);
        }

        let count = reader.u32()? as usize;
        let mut pages = HashMap::with_capacity(count);
        for _ in 0..count {
            let component_count = reader.u16()? as usize;
            let mut path = Vec::with_capacity(component_count);
            for _ in 0..component_count {
                path.push(reader.u16()?);
            }
            let len = reader.u32()? as usize;
            let bytes = reader.take(len)?;
            let text = String::from_utf8(bytes.to_vec()).map_err(|_| ParseError::NotUtf8)?;
            pages.insert(path, text);
        }

        Ok(Self { pages })
    }

    /// The prerendered text of a page, if it is part of the manual.
    pub fn page(&self, path: &[u16]) -> Option<&str> {
        self.pages.get(path).map(String::as_str)
    }

    /// Whether any page was loaded.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The blob does not start with the expected magic.
    BadMagic,
    /// The blob ends in the middle of a value.
    Truncated,
    /// A page text is not valid UTF-8.
    NotUtf8,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::BadMagic => write!(f, "not a prerendered manual blob"),
            ParseError::Truncated => write!(f, "truncated prerendered manual blob"),
            ParseError::NotUtf8 => write!(f, "manual page is not valid UTF-8"),
        }
    }
}

impl std::error::Error for ParseError {}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], ParseError> {
        let end = self.pos.checked_add(len).ok_or(ParseError::Truncated)?;
        let slice = self.data.get(self.pos..end).ok_or(ParseError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }

    fn u16(&mut self) -> Result<u16, ParseError> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn u32(&mut self) -> Result<u32, ParseError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let blob = build_blob(40);
        let manual = PrerenderedManual::parse(&blob).expect("blob parses");
        assert!(!manual.is_empty(), "the manual has pages");
        // The introduction is page 1
        assert!(manual.page(&[1]).is_some(), "page 1 is present");
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(
            PrerenderedManual::parse(b"nope").unwrap_err(),
            ParseError::BadMagic
        );
        assert_eq!(
            PrerenderedManual::parse(b"DMAN\x01\x00\x00\x00\x05").unwrap_err(),
            ParseError::Truncated
        );
    }
}
