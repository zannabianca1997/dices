//! Implementation of displays

use std::iter::once;

use either::Either::{Left, Right};
use pretty::DocBuilder;
pub use pretty::{Arena, DocAllocator, Pretty};

use crate::{
    expr::{Expr, Receiver},
    identifier::IdentStr,
    value::Value,
};

impl<'a, D, A> Pretty<'a, D, A> for &'a Expr
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A> + 'a,
    DocBuilder<'a, D, A>: Clone,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        match self {
            Expr::Null => allocator.text("null"),
            Expr::Bool(b) => allocator.text(if *b { "true" } else { "false" }),
            Expr::Number(n) => allocator.text(n.to_string()),
            Expr::List(l) => allocator
                .line_()
                .append(allocator.intersperse(l, allocator.text(",").append(allocator.line())))
                .append(allocator.line_())
                .nest(4)
                .group()
                .brackets(),
            Expr::String(s) => str_lit(allocator, s),
            Expr::Map(m) => allocator
                .line_()
                .append(allocator.intersperse(
                    m.iter().map(|(k, v)| {
                        allocator
                            .text(&**k)
                            .append(allocator.text(":"))
                            .append(allocator.space())
                            .append(v)
                    }),
                    allocator.text(",").append(allocator.line()),
                ))
                .append(allocator.line_())
                .nest(4)
                .group()
                .enclose("<|", "|>"),
            Expr::Reference(r) => r.pretty(allocator),
            Expr::Function { params, body } => allocator
                .intersperse(
                    params.iter().map(|p| &**p),
                    allocator.text(",").append(allocator.line_()),
                )
                .enclose(allocator.line_(), allocator.line_())
                .nest(4)
                .group()
                .enclose("|", "|")
                .append(allocator.space())
                .append(body.pretty(allocator)),
            Expr::Call { box fun, params } => fun.pretty(allocator).parens().append(
                allocator
                    .intersperse(&**params, allocator.text(",").append(allocator.line_()))
                    .enclose(allocator.line_(), allocator.line_())
                    .nest(4)
                    .group()
                    .parens(),
            ),
            Expr::Set { receiver, value } => receiver
                .pretty(allocator)
                .append(allocator.space())
                .append("=")
                .append(allocator.space())
                .append(value.pretty(allocator)),
            Expr::Scope(exprs) => pretty_scope(allocator, exprs),
            Expr::Sum(a) => allocator
                .intersperse(
                    a.iter().map(|a| a.pretty(allocator).parens()),
                    allocator.line().append("+").append(allocator.space()),
                )
                .group(),
            Expr::Neg(a) => allocator.text("-").append(a.pretty(allocator)),
            Expr::Mul(a, b) => a
                .pretty(allocator)
                .parens()
                .append(allocator.line())
                .append("*")
                .append(allocator.space())
                .append(b.pretty(allocator).parens())
                .group(),
            Expr::Div(a, b) => a
                .pretty(allocator)
                .parens()
                .append(allocator.line())
                .append("/")
                .append(allocator.space())
                .append(b.pretty(allocator).parens())
                .group(),
            Expr::Rem(a, b) => a
                .pretty(allocator)
                .parens()
                .append(allocator.line())
                .append("%")
                .append(allocator.space())
                .append(b.pretty(allocator).parens())
                .group(),
            Expr::Rep(a, b) => a
                .pretty(allocator)
                .parens()
                .append(allocator.line())
                .append("^")
                .append(allocator.space())
                .append(b.pretty(allocator).parens())
                .group(),
            Expr::Dice(f) => allocator.text("d").append(f.pretty(allocator).parens()),
            Expr::Join(a, b) => a
                .pretty(allocator)
                .parens()
                .append(allocator.line())
                .append("~")
                .append(allocator.space())
                .append(b.pretty(allocator).parens())
                .group(),
        }
    }
}

impl<'a, D, A> Pretty<'a, D, A> for &'a Value
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A> + 'a,
    DocBuilder<'a, D, A>: Clone,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        match self {
            Value::Null => allocator.text("null"),
            Value::Bool(b) => allocator.text(if *b { "true" } else { "false" }),
            Value::Number(n) => allocator.text(n.to_string()),
            Value::List(l) => allocator
                .intersperse(l, allocator.text(",").append(allocator.line()))
                .enclose(allocator.line_(), allocator.line_())
                .nest(4)
                .group()
                .brackets(),
            Value::String(s) => str_lit(allocator, s),
            Value::Map(m) => allocator
                .intersperse(
                    m.iter().map(|(k, v)| {
                        map_key(allocator, k)
                            .append(allocator.text(":"))
                            .append(allocator.space())
                            .append(v)
                    }),
                    allocator.text(",").append(allocator.line()),
                )
                .enclose(allocator.line_(), allocator.line_())
                .nest(4)
                .group()
                .enclose("<|", "|>"),
            Value::Function {
                params,
                context,
                body,
            } => allocator
                .intersperse(
                    params.iter().map(|p| &**p),
                    allocator.text(",").append(allocator.line_()),
                )
                .enclose(allocator.line_(), allocator.line_())
                .nest(4)
                .group()
                .enclose("|", "|")
                .append(allocator.space())
                .append({
                    if context.is_empty() {
                        // simple expression
                        body.pretty(allocator)
                    } else {
                        // complex body or non null context
                        // context is added as a series of let statements
                        let stm_docs = context
                            .iter()
                            .map(|(k, v)| pretty_let(allocator, k, Some(v)))
                            .chain(
                                if let Expr::Scope(exprs) = &**body {
                                    Right(&**exprs)
                                } else {
                                    Left(once(&**body))
                                }
                                .into_iter()
                                .map(|e| e.pretty(allocator)),
                            );
                        pretty_scope(allocator, stm_docs)
                    }
                }),
        }
    }
}

impl<'a, D, A> Pretty<'a, D, A> for &'a Receiver
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A> + 'a,
    DocBuilder<'a, D, A>: Clone,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        match self {
            Receiver::Set(var) => var.pretty(allocator),
            Receiver::Let(var) => allocator
                .text("let")
                .append(allocator.space())
                .append(var.pretty(allocator)),
            Receiver::Discard => allocator.text("_"),
        }
    }
}

/// Print a map key either as a identifier, or as a string
fn map_key<'a, D, A>(allocator: &'a D, k: &'a str) -> pretty::DocBuilder<'a, D, A>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A> + 'a,
    DocBuilder<'a, D, A>: Clone,
{
    if let Some(k) = IdentStr::new(k) {
        allocator.text(&**k)
    } else {
        str_lit(allocator, k)
    }
}

fn escape<'a, D, A>(allocator: &'a D, ch: char) -> Option<pretty::DocBuilder<'a, D, A>>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A> + 'a,
    DocBuilder<'a, D, A>: Clone,
{
    match ch {
        // special escapes that must be used whenever possible
        '\n' => Some(allocator.text("\\n")),
        '\r' => Some(allocator.text("\\r")),
        '\t' => Some(allocator.text("\\t")),
        '\0' => Some(allocator.text("\\0")),
        '\"' => Some(allocator.text("\\\"")),
        '\\' => Some(allocator.text("\\\\")),
        // char we can show literally
        _ if ch.is_alphanumeric() || ch.is_whitespace() || ch.is_ascii_graphic() => None,
        // catch all for escaping
        '\x00'..='\x7f' => Some(allocator.text(format!("\\x{:02x}", ch as u32))),
        _ => Some(allocator.text(format!("\\u{{{:x}}}", ch as u32))),
    }
}

fn str_lit<'a, D, A>(allocator: &'a D, mut s: &'a str) -> pretty::DocBuilder<'a, D, A>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A> + 'a,
    DocBuilder<'a, D, A>: Clone,
{
    let mut res = allocator.nil();

    while !s.is_empty() {
        let Some((start, end, escape)) = s.char_indices().find_map(|(pos, ch)| {
            escape(allocator, ch).map(|code| (pos, pos + ch.len_utf8(), code))
        }) else {
            // no more escape codes in the string, add the final unescaped part
            res = res.append(s);
            break;
        };
        // append unescaped part
        if start > 0 {
            res = res.append(&s[..start])
        }
        // append escape code
        res = res.append(escape);
        // cut the string
        s = &s[end..]
    }

    res.double_quotes()
}

fn pretty_scope<'a, D, A>(
    allocator: &'a D,
    stm_docs: impl IntoIterator<Item = impl Pretty<'a, D, A>>,
) -> pretty::DocBuilder<'a, D, A>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A> + 'a,
    DocBuilder<'a, D, A>: Clone,
{
    allocator
        .intersperse(stm_docs, allocator.text(";").append(allocator.line()))
        .enclose(allocator.line_(), allocator.line_())
        .nest(4)
        .group()
        .braces()
}

/// Prettify a let statement. Factored out cause it is used both in closure values and statements
fn pretty_let<'a, D, A>(
    allocator: &'a D,
    k: &'a IdentStr,
    v: Option<impl Pretty<'a, D, A>>,
) -> pretty::DocBuilder<'a, D, A>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A> + 'a,
    DocBuilder<'a, D, A>: Clone,
{
    allocator
        .text("let")
        .append(allocator.space())
        .append(k)
        .append(v.map(|v| {
            allocator
                .space()
                .append("=")
                .append(allocator.space())
                .append(v)
        }))
}

impl<'a, D, A> Pretty<'a, D, A> for &'a IdentStr
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        allocator.text(self.as_ref())
    }
}
