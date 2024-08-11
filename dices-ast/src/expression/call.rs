use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionCall {
    /// the called expression
    pub called: Box<Expression>,
    /// the params of the call
    pub params: Box<[Expression]>,
}
