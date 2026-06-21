use dices_values::string::ValueString;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiteralString(pub ValueString);
