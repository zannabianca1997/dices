use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ServerIntrisicsData {}

#[derive(Debug, Clone, Error)]
pub enum ServerIntrisicsError {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ServerIntrisics {}

impl dices_ast::intrisics::InjectedIntr for ServerIntrisics {
    type Data = ServerIntrisicsData;

    type Error = ServerIntrisicsError;

    fn iter() -> impl IntoIterator<Item = Self> {
        []
    }

    fn name(&self) -> &'static str {
        match *self {}
    }

    fn named(name: &str) -> Option<Self> {
        match name {
            _ => None,
        }
    }

    fn call(
        &self,
        _data: &mut Self::Data,
        _params: Box<[dices_ast::Value<Self>]>,
    ) -> Result<dices_ast::Value<Self>, Self::Error> {
        match *self {}
    }
}
