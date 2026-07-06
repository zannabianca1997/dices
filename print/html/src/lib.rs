#![doc = include_str!("../README.md")]

use std::io;

use html_escape::{encode_double_quoted_attribute_to_writer, encode_safe_to_writer};
use pretty::{Arena, Pretty, Render, RenderAnnotated};

use dices_print::Element;

/// Render an annotated document to an HTML fragment string.
pub fn render<'a>(
    printing: impl Pretty<'a, Arena<'a, Element>, Element>,
    arena: &'a Arena<'a, Element>,
) -> io::Result<String> {
    let mut writer = HtmlWriter::new(Vec::new());
    // usize::MAX as "do not bother with wrapping"
    printing.pretty(arena).render_raw(usize::MAX, &mut writer)?;
    Ok(String::from_utf8(writer.into_inner()).expect("HtmlWriter only ever writes valid UTF-8"))
}

/// Html writer adapter
pub struct HtmlWriter<W> {
    upstream: W,
    /// Open tags stack
    stack: Vec<&'static str>,
}

impl<W> HtmlWriter<W> {
    pub fn new(upstream: W) -> Self {
        Self {
            upstream,
            stack: Vec::new(),
        }
    }

    pub fn into_inner(self) -> W {
        self.upstream
    }
}

impl<W> Render for HtmlWriter<W>
where
    W: io::Write,
{
    type Error = io::Error;

    fn write_str(&mut self, s: &str) -> io::Result<usize> {
        encode_safe_to_writer(s, &mut self.upstream)?;
        Ok(s.len())
    }

    fn write_str_all(&mut self, s: &str) -> io::Result<()> {
        encode_safe_to_writer(s, &mut self.upstream)
    }

    fn fail_doc(&self) -> Self::Error {
        io::Error::other("Document failed to render")
    }
}

impl<W> RenderAnnotated<'_, Element> for HtmlWriter<W>
where
    W: io::Write,
{
    fn push_annotation(&mut self, element: &Element) -> io::Result<()> {
        let tag = element.local_name();

        write!(self.upstream, "<{tag}")?;
        for (name, value) in element.attributes() {
            write!(self.upstream, " {name}=\"")?;
            encode_double_quoted_attribute_to_writer(&value, &mut self.upstream)?;
            write!(self.upstream, "\"")?;
        }
        write!(self.upstream, ">")?;

        self.stack.push(tag);
        Ok(())
    }

    fn pop_annotation(&mut self) -> io::Result<()> {
        if let Some(tag) = self.stack.pop() {
            write!(self.upstream, "</{tag}>")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dices_print::MarkdownElement;
    use pretty::DocAllocator;
    use url::Url;

    use super::*;

    #[test]
    fn escapes_text_and_emits_theme_tags() {
        let arena = Arena::new();
        let url = Url::parse("https://example.com/a?x=1&y=2").unwrap();

        let doc = arena
            .text("<script>alert(1)</script>")
            .annotate(Element::Markdown(Some(MarkdownElement::Link { url })));

        let html = render(doc, &arena).unwrap();

        assert_eq!(
            html,
            "<a href=\"https://example.com/a?x=1&amp;y=2\">\
             &lt;script&gt;alert(1)&lt;&#x2F;script&gt;\
             </a>"
        );
    }
}
