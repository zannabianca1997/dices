#![doc = include_str!("../README.md")]

use std::{borrow::Cow, io};

use pretty::{Arena, Pretty, Render, RenderAnnotated};
use termcolor::{Ansi, HyperlinkSpec, WriteColor};
use url::Url;

use dices_print::{
    Element,
    theme::{FullStyle, MergeStyle, Style},
};

pub mod examples;

/// Render an annotated document to ANSI terminal output.
///
/// `style` resolves the (possibly cached) stylesheet lookup for an element,
/// and `color` toggles whether styling/hyperlinks are emitted at all.
pub fn render<'a>(
    printing: impl Pretty<'a, Arena<'a, Element>, Element>,
    arena: &'a Arena<'a, Element>,
    mut out: impl io::Write,
    width: usize,
    color: bool,
    style: impl Fn(&Element) -> Style,
) -> io::Result<()> {
    if color {
        let mut printer = PrintAnnotated::new(out, style);
        printing.pretty(arena).render_raw(width, &mut printer)
    } else {
        printing.pretty(arena).render(width, &mut out)
    }
}

/// Prompt strings needed to render `dices-example` code blocks.
pub trait PromptDisplay {
    fn prompt_left(&self) -> Cow<'static, str>;
    fn prompt_indicator(&self) -> Cow<'static, str>;
    fn prompt_multiline_indicator(&self) -> Cow<'static, str>;
}

struct PrintAnnotated<W, F> {
    upstream: W,
    style: F,
    style_stack: Vec<StyleFrame>,
}

struct StyleFrame {
    full_style: FullStyle,
    link: Option<Url>,
}

impl<W, F> PrintAnnotated<W, F> {
    fn current_style(&self) -> &FullStyle {
        &self.style_stack.last().unwrap().full_style
    }
    fn current_link(&self) -> Option<&Url> {
        self.style_stack.last().unwrap().link.as_ref()
    }
}

impl<W, F> PrintAnnotated<Ansi<W>, F>
where
    W: io::Write,
    F: Fn(&Element) -> Style,
{
    fn new(writer: W, style: F) -> Self {
        Self {
            upstream: Ansi::new(writer),
            style,
            style_stack: vec![StyleFrame {
                full_style: FullStyle::default(),
                link: None,
            }],
        }
    }
}

impl<W, F> Render for PrintAnnotated<W, F>
where
    W: io::Write,
{
    type Error = io::Error;

    fn write_str(&mut self, s: &str) -> io::Result<usize> {
        self.upstream.write(s.as_bytes())
    }

    fn write_str_all(&mut self, s: &str) -> io::Result<()> {
        self.upstream.write_all(s.as_bytes())
    }

    fn fail_doc(&self) -> Self::Error {
        io::Error::other("Document failed to render")
    }
}

impl<W, F> RenderAnnotated<'_, Element> for PrintAnnotated<W, F>
where
    W: WriteColor,
    F: Fn(&Element) -> Style,
{
    fn push_annotation(&mut self, element: &Element) -> Result<(), Self::Error> {
        let mut full_style = *self.current_style();
        (self.style)(element).apply_to(&mut full_style);

        self.upstream.set_color(&color_spec(&full_style))?;

        let link = element.url().or(self.current_link()).cloned();

        if link.as_ref() != self.current_link() {
            if self.current_link().is_some() {
                self.upstream.set_hyperlink(&HyperlinkSpec::close())?;
            }
            if let Some(link) = &link {
                self.upstream
                    .set_hyperlink(&HyperlinkSpec::open(link.as_str().as_bytes()))?;
            }
        }

        self.style_stack.push(StyleFrame { full_style, link });

        Ok(())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        let removed = self.style_stack.pop();
        let old = self.style_stack.last();

        match (old, removed) {
            (Some(old), Some(removed)) => {
                self.upstream.set_color(&color_spec(&old.full_style))?;

                if removed.link != old.link {
                    if removed.link.is_some() {
                        self.upstream.set_hyperlink(&HyperlinkSpec::close())?;
                    }
                    if let Some(link) = old.link.as_ref() {
                        self.upstream
                            .set_hyperlink(&HyperlinkSpec::open(link.as_str().as_bytes()))?;
                    }
                }

                Ok(())
            }
            (None, Some(removed)) => {
                self.upstream.reset()?;
                if removed.link.is_some() {
                    self.upstream.set_hyperlink(&HyperlinkSpec::close())?;
                }
                Ok(())
            }
            (Some(_), None) => unreachable!(),
            (None, None) => Ok(()),
        }
    }
}

/// Convert a [`dices_print`] color into a [`termcolor`] one.
pub fn convert_color(c: dices_print::theme::Color) -> termcolor::Color {
    use dices_print::theme::Color as ThemeColor;
    use termcolor::Color as TermColor;
    match c {
        ThemeColor::Black => TermColor::Black,
        ThemeColor::Blue => TermColor::Blue,
        ThemeColor::Green => TermColor::Green,
        ThemeColor::Red => TermColor::Red,
        ThemeColor::Cyan => TermColor::Cyan,
        ThemeColor::Magenta => TermColor::Magenta,
        ThemeColor::Yellow => TermColor::Yellow,
        ThemeColor::White => TermColor::White,
        ThemeColor::Ansi256(n) => TermColor::Ansi256(n),
        ThemeColor::Rgb(r, g, b) => TermColor::Rgb(r, g, b),
    }
}

fn color_spec(spec: &FullStyle) -> termcolor::ColorSpec {
    let mut cs = termcolor::ColorSpec::new();
    cs.set_fg(spec.fg_color.map(convert_color));
    cs.set_bg(spec.bg_color.map(convert_color));
    cs.set_bold(spec.bold);
    cs.set_intense(spec.intense);
    cs.set_underline(spec.underline);
    cs.set_dimmed(spec.dimmed);
    cs.set_italic(spec.italic);
    cs.set_reset(spec.reset);
    cs
}
