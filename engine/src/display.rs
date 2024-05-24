//! Implementation of displays

use pretty::{DocAllocator, DocBuilder, Pretty};

use crate::{
    expr::{Expr, Statement},
    identifier::DIdentifier,
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
            Expr::String(_) => todo!(),
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
                .intersperse(&**params, allocator.text(",").append(allocator.line_()))
                .enclose(allocator.line_(), allocator.line_())
                .nest(4)
                .group()
                .enclose("|", "|")
                .append(allocator.space())
                .append({
                    if let [Statement::Expr(e)] = &**body {
                        // simple expression
                        e.pretty(allocator)
                    } else {
                        // complex body
                        pretty_scope(allocator, body.iter())
                    }
                }),
            Expr::Call { box fun, params } => {
                let fun_doc = fun.pretty(allocator);
                let fun_doc = if fun.need_parents_for_call() {
                    fun_doc.parens()
                } else {
                    fun_doc
                };
                fun_doc.append(
                    allocator
                        .intersperse(&**params, allocator.text(",").append(allocator.line_()))
                        .enclose(allocator.line_(), allocator.line_())
                        .nest(4)
                        .group()
                        .parens(),
                )
            }
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
            Value::String(_) => todo!(),
            Value::Map(m) => allocator
                .intersperse(
                    m.iter().map(|(k, v)| {
                        allocator
                            .text(&**k)
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
                .intersperse(&**params, allocator.text(",").append(allocator.line_()))
                .enclose(allocator.line_(), allocator.line_())
                .nest(4)
                .group()
                .enclose("|", "|")
                .append(allocator.space())
                .append({
                    if let (true, [Statement::Expr(e)]) = (context.is_empty(), &**body) {
                        // simple expression
                        e.pretty(allocator)
                    } else {
                        // complex body or non null context
                        // context is added as a series of let statements
                        let stm_docs = context
                            .iter()
                            .map(|(k, v)| pretty_let(allocator, k, Some(v)))
                            .chain(body.iter().map(|s| s.pretty(allocator)));
                        pretty_scope(allocator, stm_docs)
                    }
                }),
        }
    }
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
    k: &'a DIdentifier,
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

impl<'a, D, A> Pretty<'a, D, A> for &'a Statement
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A> + 'a,
    DocBuilder<'a, D, A>: Clone,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        match self {
            Statement::Expr(e) => e.pretty(allocator),
            Statement::Set(n, v) => n
                .pretty(allocator)
                .append(allocator.space())
                .append("=")
                .append(allocator.space())
                .append(v),
            Statement::Let(n, v) => pretty_let(allocator, n, v.as_ref()),
            Statement::Scope(body) => pretty_scope(allocator, body.iter()),
        }
    }
}

impl<'a, D, A> Pretty<'a, D, A> for &'a DIdentifier
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        allocator.text(self.as_str())
    }
}
