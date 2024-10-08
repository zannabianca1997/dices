//! Expression to read the members of a composite

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode,),
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr")
)]
/// Access a member of a map or a list
pub struct ExpressionMemberAccess<InjectedIntrisic> {
    pub accessed: Box<Expression<InjectedIntrisic>>,
    pub index: Box<Expression<InjectedIntrisic>>,
}
