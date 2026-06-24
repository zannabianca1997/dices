use pretty::{DocAllocator, DocBuilder, Pretty};

use crate::{AstElement, DelimiterKind, Element, ValueElement};
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

///
pub struct Ctx {

    /// Indentation used when a container is broken over multiple lines.
    indent: usize
}

/// A bracket / parenthesis token of the given `kind` at the given nesting `depth`.
fn delim<'a, D>(
    alloc: &'a D,
    text: &'a str,
    kind: DelimiterKind,
    depth: u8,
) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
{
    alloc
        .text(text)
        .annotate(Element::Value(Some(ValueElement::Delimiter {
            kind,
            depth,
        })))
}

/// A punctuator token (`,`, `:`, ...).
fn punct<'a, D>(alloc: &'a D, text: &'a str) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
{
    alloc
        .text(text)
        .annotate(Element::Value(Some(ValueElement::Punctuator)))
}

/// A trailing comma that appears only when a container is broken over
/// multiple lines (nothing when it stays flat).
fn trailing_comma<'a, D>(alloc: &'a D) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
{
    punct(alloc, ",").flat_alt(alloc.nil())
}

/// Render any [`Value`], threading the current bracket nesting `depth`.
///
/// This is the single point of recursion: leaf [`Pretty`] impls and the
/// container helpers all route through here so that `depth` keeps climbing as
/// we descend into nested lists and maps.
fn value_doc<'a, D>(value: Value, alloc: &'a D, depth: u8) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
    D::Doc: Clone,
{
    match value {
        Value::Null(v) => null_doc(v, alloc),
        Value::Bool(v) => bool_doc(v, alloc),
        Value::Int(v) => int_doc(v, alloc),
        Value::String(v) => string_doc(v, alloc),
        Value::List(v) => list_doc(v, alloc, depth),
        Value::Map(v) => map_doc(v, alloc, depth),
        Value::Injected(v) => injected_doc(v, alloc),
    }
}

fn null_doc<'a, D>(_value: ValueNull, alloc: &'a D) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
{
    alloc
        .text("null")
        .annotate(Element::Value(Some(ValueElement::Null)))
}

fn bool_doc<'a, D>(value: ValueBool, alloc: &'a D) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
{
    alloc
        .text(if value.get() { "true" } else { "false" })
        .annotate(Element::Value(Some(ValueElement::Bool {
            value: value.get(),
        })))
}

fn int_doc<'a, D>(value: ValueInt, alloc: &'a D) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
{
    alloc
        .text(value.to_string())
        .annotate(Element::Value(Some(ValueElement::Integer)))
}

fn string_doc<'a, D>(value: ValueString, alloc: &'a D) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
{
    let string_elem = |escape| Element::Value(Some(ValueElement::String { escape }));
    let escape = Escape::default();

    // Group consecutive chars into runs of equal escaped-ness, so each
    // escape sequence becomes its own annotated fragment.
    let mut runs: Vec<(bool, String)> = Vec::new();
    for ch in value.as_str().chars() {
        let escaped = escape.escapes(ch);
        if runs.last().map(|(e, _)| *e) != Some(escaped) {
            runs.push((escaped, String::new()));
        }
        let buf = &mut runs.last_mut().expect("just pushed if empty").1;
        match escape.escape_char(ch) {
            Some(repr) => buf.push_str(&repr),
            None => buf.push(ch),
        }
    }

    let content = alloc.concat(
        runs.into_iter()
            .map(|(escaped, text)| alloc.text(text).annotate(string_elem(escaped))),
    );

    alloc
        .text("\"")
        .annotate(string_elem(false))
        .append(content)
        .append(alloc.text("\"").annotate(string_elem(false)))
}

fn string_or_ident_doc<'a, D>(value: ValueString, alloc: &'a D) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
{
    match Identifier::new(value) {
        Ok(value) => ident_doc(value, alloc),
        Err(value) => string_doc(value, alloc),
    }
}

fn ident_doc<'a, D>(value: Identifier, alloc: &'a D) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
{
    alloc
        .text(String::from(value))
        .annotate(Element::Ast(Some(AstElement::Ident)))
}

fn list_doc<'a, D>(value: ValueList, alloc: &'a D, depth: u8) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
    D::Doc: Clone,
{
    let open = delim(alloc, "[", DelimiterKind::List, depth);
    let close = delim(alloc, "]", DelimiterKind::List, depth);

    if value.is_empty() {
        return open.append(close);
    }

    let items = alloc.intersperse(
        value
            .into_iter()
            .map(|el| value_doc(el, alloc, depth.wrapping_add(1))),
        punct(alloc, ",").append(alloc.line()),
    );

    open.append(
        alloc
            .line_()
            .append(items)
            .append(trailing_comma(alloc))
            .nest(INDENT),
    )
    .append(alloc.line_())
    .append(close)
    .group()
}

fn map_doc<'a, D>(value: ValueMap, alloc: &'a D, depth: u8) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
    D::Doc: Clone,
{
    let open = delim(alloc, "<|", DelimiterKind::Map, depth);
    let close = delim(alloc, "|>", DelimiterKind::Map, depth);

    if value.is_empty() {
        return open.append(close);
    }

    let entries = alloc.intersperse(
        value.into_iter().map(|(key, val)| {
            string_or_ident_doc(key, alloc)
                .append(punct(alloc, ":"))
                .append(alloc.space())
                .append(value_doc(val, alloc, depth.wrapping_add(1)))
        }),
        punct(alloc, ",").append(alloc.line()),
    );

    open.append(
        alloc
            .line()
            .append(entries)
            .append(trailing_comma(alloc))
            .nest(INDENT),
    )
    .append(alloc.line())
    .append(close)
    .group()
}

fn injected_doc<'a, D>(value: ValueInjected, alloc: &'a D) -> DocBuilder<'a, D, Element>
where
    D: DocAllocator<'a, Element>,
{
    alloc
        .text(format!("<{}>", value.description()))
        .annotate(Element::Value(Some(ValueElement::Injected)))
}

impl<'a, D> Pretty<'a, D, Element> for ValueNull
where
    D: DocAllocator<'a, Element>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Element> {
        null_doc(self, allocator)
    }
}

impl<'a, D> Pretty<'a, D, Element> for ValueBool
where
    D: DocAllocator<'a, Element>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Element> {
        bool_doc(self, allocator)
    }
}

impl<'a, D> Pretty<'a, D, Element> for ValueInt
where
    D: DocAllocator<'a, Element>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Element> {
        int_doc(self, allocator)
    }
}

impl<'a, D> Pretty<'a, D, Element> for ValueString
where
    D: DocAllocator<'a, Element>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Element> {
        string_doc(self, allocator)
    }
}

impl<'a, D> Pretty<'a, D, Element> for ValueList
where
    D: DocAllocator<'a, Element>,
    D::Doc: Clone,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Element> {
        list_doc(self, allocator, 0)
    }
}

impl<'a, D> Pretty<'a, D, Element> for ValueMap
where
    D: DocAllocator<'a, Element>,
    D::Doc: Clone,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Element> {
        let depth = 0;
        let open = delim(allocator, "<|", DelimiterKind::Map, depth);
        let close = delim(allocator, "|>", DelimiterKind::Map, depth);

        if self.is_empty() {
            return open.append(close);
        }

        let entries = allocator.intersperse(
            self.into_iter().map(|(key, val)| {
                string_or_ident_doc(key, allocator)
                    .append(punct(allocator, ":"))
                    .append(allocator.space())
                    .append(value_doc(val, allocator, depth.wrapping_add(1)))
            }),
            punct(allocator, ",").append(allocator.line()),
        );

        open.append(
            allocator
                .line()
                .append(entries)
                .append(trailing_comma(allocator))
                .nest(INDENT),
        )
        .append(allocator.line())
        .append(close)
        .group()
    }
}

impl<'a, D> Pretty<'a, D, Element> for ValueInjected
where
    D: DocAllocator<'a, Element>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Element> {
        let value = self;
        allocator
            .text(format!("<{}>", value.description()))
            .annotate(Element::Value(Some(ValueElement::Injected)))
    }
}

impl<'a, D> Pretty<'a, D, Element> for Value
where
    D: DocAllocator<'a, Element>,
    D::Doc: Clone,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Element> {
        let depth = 0;
        match self {
            Value::Null(v) => v.pretty(allocator),
            Value::Bool(v) => v.pretty(allocator),
            Value::Int(v) => v.pretty(allocator),
            Value::String(v) => v.pretty(allocator),
            Value::List(v) => v.pretty(allocator),
            Value::Map(v) => v.pretty(allocator),
            Value::Injected(v) => v.pretty(allocator),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use pretty::{Arena, Render, RenderAnnotated};

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
        value.pretty(&arena).render_fmt(width, &mut out).unwrap();
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
                Some(ValueElement::Delimiter { depth, .. }) => Some((text, depth)),
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
