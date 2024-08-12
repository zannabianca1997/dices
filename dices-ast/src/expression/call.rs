use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionCall {
    /// the called expression
    pub called: Box<Expression>,
    /// the params of the call
    pub params: Box<[Expression]>,
}

impl ExpressionCall {
    pub fn new(called: Expression, params: Box<[Expression]>) -> Self {
        Self {
            called: Box::new(called),
            params,
        }
    }
}
