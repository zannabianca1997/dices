use dices_values::bool::ValueBool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiteralBool(pub ValueBool);
