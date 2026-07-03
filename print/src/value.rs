use crate::{AstElement, DelimiterKind, DocAllocator, DocBuilder, Element, Pretty, ValueElement};
use dices_values::{
    Value,
    bool::ValueBool,
    identifier::Identifier,
    injected::ValueInjected,
    int::ValueInt,
    list::ValueList,
    map::ValueMap,
    null::ValueNull,
    string::{Escape, ValueString},
};

/// Context for printing a value
#[derive(Debug, Clone, Copy)]
pub struct Ctx {
    /// Nesting level
    nesting: u8,
    /// Indentation used when a container is broken over multiple lines.
    indent: isize,
    /// Escape strategy for strings
    escape: Escape,
}
impl Ctx {
    pub fn new(indent: u8, escape: Escape) -> Self {
        Self {
            nesting: 0,
            indent: indent as _,
            escape,
        }
    }

    fn nested(&self) -> Self {
        Self {
            nesting: self.nesting.wrapping_add(1),
            ..*self
        }
    }
}
impl Default for Ctx {
    fn default() -> Self {
        Self::new(2, Escape::default())
    }
}

/// A bracket / parenthesis token of the given `kind` at the given nesting `depth`.
fn delim<'a, D>(alloc: &'a D, text: &'a str, kind: DelimiterKind, nesting: u8) -> DocBuilder<'a, D>
where
    D: DocAllocator<'a>,
{
    alloc
        .text(text)
        .annotate(Element::Value(Some(ValueElement::Delimiter {
            kind,
            nesting,
        })))
}

/// A punctuator token (`,`, `:`, ...).
fn punct<'a, D>(alloc: &'a D, text: &'a str) -> DocBuilder<'a, D>
where
    D: DocAllocator<'a>,
{
    alloc
        .text(text)
        .annotate(Element::Value(Some(ValueElement::Punctuator)))
}

/// A trailing comma that appears only when a container is broken over
/// multiple lines (nothing when it stays flat).
fn trailing_comma<'a, D>(alloc: &'a D) -> DocBuilder<'a, D>
where
    D: DocAllocator<'a>,
{
    punct(alloc, ",").flat_alt(alloc.nil())
}

fn string_or_ident_doc<'a, D>(
    value: &'a ValueString,
    allocator: &'a D,
    ctx: &mut Ctx,
) -> DocBuilder<'a, D>
where
    D: DocAllocator<'a>,
{
    match Identifier::new_ref(value) {
        Some(value) => ident_doc(value, allocator, ctx),
        None => value.pretty(allocator, ctx),
    }
}

fn ident_doc<'a, D>(value: &'a Identifier, allocator: &'a D, _ctx: &mut Ctx) -> DocBuilder<'a, D>
where
    D: DocAllocator<'a>,
{
    allocator
        .text(value.as_ref().as_str())
        .annotate(Element::Ast(Some(AstElement::Ident)))
}

impl<'a, D> Pretty<'a, D, Ctx> for &'a ValueNull
where
    D: DocAllocator<'a>,
{
    fn pretty(self, allocator: &'a D, _ctx: &mut Ctx) -> DocBuilder<'a, D> {
        allocator
            .text("null")
            .annotate(Element::Value(Some(ValueElement::Null)))
    }
}

impl<'a, D> Pretty<'a, D, Ctx> for &'a ValueBool
where
    D: DocAllocator<'a>,
{
    fn pretty(self, allocator: &'a D, _ctx: &mut Ctx) -> DocBuilder<'a, D> {
        allocator
            .text(if self.get() { "true" } else { "false" })
            .annotate(Element::Value(Some(ValueElement::Bool {
                value: self.get(),
            })))
    }
}

impl<'a, D> Pretty<'a, D, Ctx> for &'a ValueInt
where
    D: DocAllocator<'a>,
{
    fn pretty(self, allocator: &'a D, _ctx: &mut Ctx) -> DocBuilder<'a, D> {
        allocator
            .text(self.to_string())
            .annotate(Element::Value(Some(ValueElement::Integer)))
    }
}

impl<'a, D> Pretty<'a, D, Ctx> for &'a ValueString
where
    D: DocAllocator<'a>,
{
    fn pretty(self, allocator: &'a D, ctx: &mut Ctx) -> DocBuilder<'a, D> {
        let string_elem = |escape| Element::Value(Some(ValueElement::String { escape }));

        let runs = ctx.escape.escape_str(self);

        let content = allocator.concat(
            runs.into_iter()
                .map(|(escaped, text)| allocator.text(text).annotate(string_elem(escaped))),
        );

        allocator
            .text("\"")
            .annotate(string_elem(false))
            .append(content)
            .append(allocator.text("\"").annotate(string_elem(false)))
    }
}

impl<'a, D> Pretty<'a, D, Ctx> for &'a ValueList
where
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    fn pretty(self, allocator: &'a D, ctx: &mut Ctx) -> DocBuilder<'a, D> {
        let open = delim(allocator, "[", DelimiterKind::List, ctx.nesting);
        let close = delim(allocator, "]", DelimiterKind::List, ctx.nesting);
        let mut ctx = ctx.nested();

        allocator
            .intersperse(
                self.iter().map(|el| el.pretty(allocator, &mut ctx)),
                punct(allocator, ",").append(allocator.line()),
            )
            .append(trailing_comma(allocator))
            .enclose(allocator.line_(), allocator.line_())
            .nest(ctx.indent)
            .enclose(open, close)
            .group()
    }
}

impl<'a, D> Pretty<'a, D, Ctx> for &'a ValueMap
where
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    fn pretty(self, allocator: &'a D, ctx: &mut Ctx) -> DocBuilder<'a, D> {
        let open = delim(allocator, "<|", DelimiterKind::Map, ctx.nesting);
        let close = delim(allocator, "|>", DelimiterKind::Map, ctx.nesting);
        let mut ctx = ctx.nested();

        allocator
            .intersperse(
                self.iter().map(|(key, val)| {
                    string_or_ident_doc(key, allocator, &mut ctx)
                        .append(punct(allocator, ":"))
                        .append(allocator.space())
                        .append(val.pretty(allocator, &mut ctx))
                }),
                punct(allocator, ",").append(allocator.line()),
            )
            .append(trailing_comma(allocator))
            .enclose(allocator.line_(), allocator.line_())
            .nest(ctx.indent)
            .enclose(open, close)
            .group()
    }
}

impl<'a, D> Pretty<'a, D, Ctx> for &'a ValueInjected
where
    D: DocAllocator<'a>,
{
    fn pretty(self, allocator: &'a D, _ctx: &mut Ctx) -> DocBuilder<'a, D> {
        allocator
            .text(format!("<{}>", self.description()))
            .annotate(Element::Value(Some(ValueElement::Injected)))
    }
}

impl<'a, D> Pretty<'a, D, Ctx> for &'a Value
where
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    fn pretty(self, allocator: &'a D, ctx: &mut Ctx) -> DocBuilder<'a, D> {
        match self {
            Value::Null(v) => v.pretty(allocator, ctx),
            Value::Bool(v) => v.pretty(allocator, ctx),
            Value::Int(v) => v.pretty(allocator, ctx),
            Value::String(v) => v.pretty(allocator, ctx),
            Value::List(v) => v.pretty(allocator, ctx),
            Value::Map(v) => v.pretty(allocator, ctx),
            Value::Injected(v) => v.pretty(allocator, ctx),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use pretty::{Arena, Pretty as _, Render, RenderAnnotated};

    use super::*;

    fn int(n: &str) -> Value {
        Value::Int(n.parse().unwrap())
    }

    fn string(s: &str) -> Value {
        Value::String(ValueString::new(s.to_owned()))
    }

    fn list(items: Vec<Value>) -> Value {
        Value::List(ValueList::new(items))
    }

    fn map(entries: Vec<(&str, Value)>) -> Value {
        Value::Map(ValueMap::new(
            entries
                .into_iter()
                .map(|(k, v)| (ValueString::new(k.to_owned()), v))
                .collect::<BTreeMap<_, _>>(),
        ))
    }

    /// Render to plain text at the given width.
    fn plain(value: Value, width: usize) -> String {
        let arena: Arena<Element> = Arena::new();
        let mut out = String::new();
        value
            .with_default_ctx()
            .pretty(&arena)
            .render_fmt(width, &mut out)
            .unwrap();
        out
    }

    #[test]
    fn leaves_match_display() {
        assert_eq!(plain(Value::default(), 80), "null");
        assert_eq!(plain(Value::Bool(true.into()), 80), "true");
        assert_eq!(plain(Value::Bool(false.into()), 80), "false");
        assert_eq!(plain(int("42"), 80), "42");
        assert_eq!(
            plain(int("-99999999999999999999"), 80),
            "-99999999999999999999"
        );
    }

    #[test]
    fn strings_escape() {
        // Matches `Display`: quoted, control chars escaped.
        assert_eq!(plain(string("a\nb"), 80), r#""a\nb""#);
        assert_eq!(plain(string("plain"), 80), r#""plain""#);
        // Quotes and backslashes escape so the literal round-trips.
        assert_eq!(plain(string("a\"b"), 80), r#""a\"b""#);
        assert_eq!(plain(string(r"a\b"), 80), r#""a\\b""#);
    }

    #[test]
    fn containers_flat_when_they_fit() {
        assert_eq!(plain(list(vec![]), 80), "[]");
        assert_eq!(
            plain(list(vec![int("1"), int("2"), int("3")]), 80),
            "[1, 2, 3]"
        );

        assert_eq!(plain(map(vec![]), 80), "<||>");
        // Maps keep inner padding when flat: `<| k: v |>`.
        assert_eq!(plain(map(vec![("a", int("1"))]), 80), "<| a: 1 |>");
    }

    #[test]
    fn containers_wrap_and_indent_when_wide() {
        let value = list(vec![int("1"), int("2"), int("3")]);
        assert_eq!(plain(value, 4), "[\n    1,\n    2,\n    3,\n]");
    }

    enum Event {
        Text(String),
        Push(Element),
        Pop,
    }

    #[derive(Default)]
    struct Collector {
        events: Vec<Event>,
    }

    impl Render for Collector {
        type Error = ();

        fn write_str(&mut self, s: &str) -> Result<usize, ()> {
            self.events.push(Event::Text(s.to_owned()));
            Ok(s.len())
        }

        fn write_str_all(&mut self, s: &str) -> Result<(), ()> {
            self.events.push(Event::Text(s.to_owned()));
            Ok(())
        }

        fn fail_doc(&self) {}
    }

    impl<'a> RenderAnnotated<'a, Element> for Collector {
        fn push_annotation(&mut self, a: &'a Element) -> Result<(), ()> {
            self.events.push(Event::Push(*a));
            Ok(())
        }

        fn pop_annotation(&mut self) -> Result<(), ()> {
            self.events.push(Event::Pop);
            Ok(())
        }
    }

    /// Collect `(innermost value element, rendered text)` for every fragment.
    fn fragments(value: Value, width: usize) -> Vec<(Option<ValueElement>, String)> {
        let arena: Arena<Element> = Arena::new();
        let mut collector = Collector::default();
        value
            .with_default_ctx()
            .pretty(&arena)
            .render_raw(width, &mut collector)
            .unwrap();

        let mut stack: Vec<Option<ValueElement>> = Vec::new();
        let mut out = Vec::new();
        for event in &collector.events {
            match event {
                Event::Push(Element::Value(element)) => stack.push(*element),
                Event::Push(_) => stack.push(None),
                Event::Pop => {
                    stack.pop();
                }
                Event::Text(text) if !text.trim().is_empty() => {
                    out.push((stack.last().copied().flatten(), text.clone()))
                }
                Event::Text(_) => {}
            }
        }
        out
    }

    #[test]
    fn delimiter_depth_climbs_with_nesting() {
        let value = list(vec![list(vec![int("1")])]);
        let depths: Vec<_> = fragments(value, 80)
            .into_iter()
            .filter_map(|(el, text)| match el {
                Some(ValueElement::Delimiter { nesting: depth, .. }) => Some((text, depth)),
                _ => None,
            })
            .collect();

        assert_eq!(
            depths,
            vec![
                ("[".to_owned(), 0),
                ("[".to_owned(), 1),
                ("]".to_owned(), 1),
                ("]".to_owned(), 0),
            ]
        );
    }

    #[test]
    fn escape_runs_are_flagged() {
        let frags = fragments(string("a\nb"), 80);
        // The `\n` fragment is the only one flagged as an escape.
        let escapes: Vec<_> = frags
            .iter()
            .filter_map(|(el, text)| match el {
                Some(ValueElement::String { escape: true }) => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(escapes, vec![r"\n"]);
    }
}
