use super::*;

impl Solvable for ExpressionUnOp {
    type Error = SolveError;

    fn solve<R>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        todo!()
    }
}
