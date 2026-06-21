use dices_values::null::ValueNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiteralNull(pub ValueNull);
