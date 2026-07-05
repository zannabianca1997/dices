//! Provide pages from the

use std::borrow::Cow;

use linkme::DistributedSlice;
pub use linkme::distributed_slice;

use crate::{ManPageContent, PathComponent, registry::Provider};

use super::DynamicProvider;

#[distributed_slice]
pub static LINKED_PAGES: [LinkedPage];

/// Page added via static linking
///
/// add these pages to the [`LINKED_PAGES`] distributed slices and they will
/// show up in the global manual
/// ```
/// use dices_man::registry::linked::*;
///
/// #[distributed_slice(LINKED_PAGES)]
/// static PAGE: LinkedPage = LinkedPage {
///     path: &[99,99,10],
///     title: "Test Linked Page",
///     content: "Hello World!"
/// };
///
/// let manual = dices_man::Manual::new();
/// assert_eq!(manual.fetch(&[99,99,10]).unwrap().title(), "Test Linked Page");
/// ```
pub struct LinkedPage {
    pub path: &'static [PathComponent],
    pub title: &'static str,
    pub content: &'static str,
}

impl Provider for &'static DistributedSlice<[LinkedPage]> {
    fn as_static(
        self,
    ) -> Result<
        impl IntoIterator<Item = (Cow<'static, [PathComponent]>, ManPageContent)>,
        Box<dyn DynamicProvider>,
    > {
        Ok(self.into_iter().map(
            |LinkedPage {
                 path,
                 title,
                 content,
             }| {
                (
                    Cow::Borrowed(*path),
                    ManPageContent {
                        title: Cow::Borrowed(title),
                        content: Cow::Borrowed(content),
                    },
                )
            },
        ))
    }
}
