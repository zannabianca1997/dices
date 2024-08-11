use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionScope {
    /// All the inner expressions except the last
    pub exprs: Box<[Expression]>,
    /// The last expression, and the one returned
    pub last: Box<Expression>,
}
