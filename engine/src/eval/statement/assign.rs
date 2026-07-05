use dices_ast::statement::assign::{AssignStatement, Lhs, MemberAccessLhs};
use dices_values::{Value, cast::push_down_if_injected};
use snafu::OptionExt;

use crate::eval::expr::member_access::{assign_member, read_member};
use crate::{EvalError, VariableDoNotExistsSnafu, context::Context, var_use::VarUse};

pub(super) fn eval(
    stmt: &AssignStatement,
    cx: &mut (impl Context + ?Sized),
) -> Result<(), EvalError> {
    let (AssignStatement::Let { rhs, .. } | AssignStatement::Set { rhs, .. }) = stmt;
    let rhs = crate::eval::expr::eval(rhs, cx)?;

    match stmt {
        AssignStatement::Let { lhs: ident, rhs: _ } => {
            cx.let_var(ident.clone(), rhs);
            Ok(())
        }
        AssignStatement::Set { lhs, rhs: _ } => assign(lhs, rhs, cx),
    }
}

/// Write `rhs` into the slot described by `lhs`.
///
/// For a plain variable this mutates the variable slot in the context. For a
/// member access chain like `a.b.c[2] = x`, the path is walked exactly once:
/// each index expression is evaluated exactly once (outer-to-inner,
/// interleaved with reads), the resulting values are cached, and the cached
/// values are reused on the way back up when rebuilding the containers.
///
/// This matters for side-effecting indices: `a[d6][2] = null` rolls the d6
/// once, not twice.
fn assign(lhs: &Lhs, rhs: Value, cx: &mut (impl Context + ?Sized)) -> Result<(), EvalError> {
    // Fast path: plain variable assignment, no member access chain.
    if let Lhs::Variable(ident) = lhs {
        let slot = cx
            .var_mut(ident)
            .with_context(|| VariableDoNotExistsSnafu {
                name: ident.clone(),
            })?;
        *slot = rhs;
        return Ok(());
    }

    // Collect the chain of member accesses from the lhs, outer-to-inner.
    // For `a.b.c[2]` this yields `[b, c, 2]` with `a` as the root variable.
    let mut chain: Vec<&dices_ast::expr::Expr> = Vec::new();
    let mut current = lhs;
    let root_ident = loop {
        match current {
            Lhs::Variable(ident) => break ident,
            Lhs::MemberAccess(MemberAccessLhs { container, index }) => {
                chain.push(index);
                current = container;
            }
        }
    };
    // The collection above is inner-to-outer; reverse for outer-to-inner so
    // that index evaluation order matches left-to-right reading order.
    chain.reverse();

    // Load the root variable's current value.
    let root_val = cx
        .var(root_ident)
        .cloned()
        .with_context(|| VariableDoNotExistsSnafu {
            name: root_ident.clone(),
        })?;

    // Descend: evaluate each index once, reading to advance down to the
    // parent of the deepest level. The deepest index is evaluated but NOT
    // used to read — the read there would be discarded by the upcoming
    // assign, so we skip it (also avoids spurious errors on missing keys).
    let mut containers: Vec<Value> = Vec::with_capacity(chain.len());
    containers.push(root_val);
    let mut indices: Vec<Value> = Vec::with_capacity(chain.len());
    for (i, index_expr) in chain.iter().enumerate() {
        let index = push_down_if_injected(crate::eval::expr::eval(index_expr, cx)?)?;
        if i + 1 < chain.len() {
            let parent = containers.last().unwrap().clone();
            let child = read_member(parent, index.clone())?;
            containers.push(child);
        }
        indices.push(index);
    }

    // Apply the rhs at the deepest level: assign_member(parent, deepest_index, rhs).
    let parent = containers.pop().unwrap();
    let deepest_index = indices.pop().unwrap();
    let mut new_value = assign_member(parent, deepest_index, rhs)?;

    // Walk back up, rebuilding each ancestor container with its cached index.
    for index in indices.into_iter().rev() {
        let container = containers.pop().unwrap();
        new_value = assign_member(container, index, new_value)?;
    }

    // Write the rebuilt root value back into the variable.
    let slot = cx
        .var_mut(root_ident)
        .with_context(|| VariableDoNotExistsSnafu {
            name: root_ident.clone(),
        })?;
    *slot = new_value;
    Ok(())
}

pub(super) fn var_use(stmt: &AssignStatement) -> VarUse {
    let (AssignStatement::Let { rhs, .. } | AssignStatement::Set { rhs, .. }) = stmt;
    let rhs = crate::eval::expr::var_use(rhs);

    match stmt {
        AssignStatement::Let { lhs, rhs: _ } => rhs.then(VarUse::r#let(lhs.clone())),
        AssignStatement::Set { lhs, rhs: _ } => rhs.then(assign_var_use(lhs)),
    }
}

/// Variable usage of an assignment to `lhs`.
///
/// Mirrors the single-descent structure of [`assign`]: the root variable is
/// read (to load the container) and then set (to write back the rebuilt
/// value), and each index expression is evaluated exactly once, outer-to-
/// inner, in between. The rhs has already been sequenced before this by the
/// caller.
fn assign_var_use(lhs: &Lhs) -> VarUse {
    // Fast path: plain `a = x` only sets `a` (no read of its prior value).
    if let Lhs::Variable(ident) = lhs {
        return VarUse::set(ident.clone());
    }

    // Collect the member-access chain outer-to-inner, locating the root.
    // For `a.b.c[2]` this yields `[b, c, 2]` with `a` as the root variable.
    let mut chain: Vec<&dices_ast::expr::Expr> = Vec::new();
    let mut current = lhs;
    let root_ident = loop {
        match current {
            Lhs::Variable(ident) => break ident,
            Lhs::MemberAccess(MemberAccessLhs { container, index }) => {
                chain.push(index);
                current = container;
            }
        }
    };
    // Collection is inner-to-outer; reverse to match evaluation order.
    chain.reverse();

    // Runtime order: read root, evaluate each index once (outer-to-inner),
    // then set root. Intermediate member reads/writes don't touch variables.
    let mut usage = VarUse::read(root_ident.clone());
    for index_expr in &chain {
        usage = usage.then(crate::eval::expr::var_use(index_expr));
    }
    usage.then(VarUse::set(root_ident.clone()))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use dices_ast::{
        expr::Expr,
        identifier::Identifier,
        statement::assign::{AssignStatement, Lhs, MemberAccessLhs},
    };
    use dices_values::string::ValueString;

    use super::var_use;

    fn ident(s: &'static str) -> Identifier {
        Identifier::new(ValueString::new_static(s)).unwrap()
    }

    fn var_expr(s: &'static str) -> Expr {
        Expr::Variable(Box::new(ident(s)))
    }

    fn member(container: Lhs, index: Expr) -> Lhs {
        Lhs::MemberAccess(MemberAccessLhs {
            container: Box::new(container),
            index,
        })
    }

    fn set(lhs: Lhs, rhs: Expr) -> AssignStatement {
        AssignStatement::Set { lhs, rhs }
    }

    fn reads(stmt: &AssignStatement) -> BTreeSet<Identifier> {
        var_use(stmt).reads
    }
    fn sets(stmt: &AssignStatement) -> BTreeSet<Identifier> {
        var_use(stmt).sets
    }

    #[test]
    fn plain_variable_set_is_not_read() {
        // `a = b`: rhs reads b, lhs sets a (no read of a's prior value).
        let stmt = set(Lhs::Variable(ident("a")), var_expr("b"));
        assert_eq!(reads(&stmt), [ident("b")].into_iter().collect());
        assert_eq!(sets(&stmt), [ident("a")].into_iter().collect());
    }

    #[test]
    fn member_access_root_is_read_and_set() {
        // `a.x = b`: a is read (to load container) and set (to write back);
        // x is a literal string key (no variable read); b is read as rhs.
        let stmt = set(
            member(Lhs::Variable(ident("a")), var_expr("b")),
            var_expr("c"),
        );
        assert_eq!(
            reads(&stmt),
            [ident("a"), ident("b"), ident("c")].into_iter().collect()
        );
        assert_eq!(sets(&stmt), [ident("a")].into_iter().collect());
    }

    #[test]
    fn nested_chain_root_read_and_set_indices_read_once() {
        // `a[i][j] = c`: a read+set, i and j read once each, c read.
        // (VarUse is a set, so "once" is the natural result; this also
        // guards against the old double-descent which sequenced indices in
        // the wrong order.)
        let stmt = set(
            member(
                member(Lhs::Variable(ident("a")), var_expr("i")),
                var_expr("j"),
            ),
            var_expr("c"),
        );
        assert_eq!(
            reads(&stmt),
            [ident("a"), ident("i"), ident("j"), ident("c")]
                .into_iter()
                .collect()
        );
        assert_eq!(sets(&stmt), [ident("a")].into_iter().collect());
    }

    #[test]
    fn root_read_survives_even_when_rhs_sets_it() {
        // `a[i] = a`: rhs reads a, lhs reads a then sets a. The set doesn't
        // cancel the earlier read (the read happens before the set at runtime).
        let stmt = set(
            member(Lhs::Variable(ident("a")), var_expr("i")),
            var_expr("a"),
        );
        assert_eq!(
            reads(&stmt),
            [ident("a"), ident("i")].into_iter().collect()
        );
        assert_eq!(sets(&stmt), [ident("a")].into_iter().collect());
    }
}
