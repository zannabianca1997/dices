use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionCall<InjectedIntrisic> {
    /// the called expression
    pub called: Box<Expression<InjectedIntrisic>>,
    /// the params of the call
    pub params: Box<[Expression<InjectedIntrisic>]>,
}

impl<InjectedIntrisic> ExpressionCall<InjectedIntrisic> {
    pub fn new(
        called: Expression<InjectedIntrisic>,
        params: Box<[Expression<InjectedIntrisic>]>,
    ) -> Self {
        Self {
            called: Box::new(called),
            params,
        }
    }
}
