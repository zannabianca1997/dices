//! Pages embedded from the `pages` directory

use std::{borrow::Cow, marker::PhantomData};

use itertools::Itertools;
use rust_embed::Embed;

use crate::{ManPageContent, PathComponent, registry::Provider};

use super::DynamicProvider;

#[derive(Debug, Embed)]
#[folder = "$CARGO_MANIFEST_DIR/pages"]
pub(super) struct EmbeddedPages;

pub struct EmbeddedPagesProvider<E, const STATIC: bool = { cfg!(not(debug_assertions)) }>(
    PhantomData<E>,
);

impl<E> EmbeddedPagesProvider<E> {
    pub fn new(_: E) -> Self
    where
        E: Embed + 'static + Send + Sync,
    {
        Self(PhantomData)
    }
}
impl<E> EmbeddedPagesProvider<E, true> {
    pub fn new_static(_: E) -> Self
    where
        E: Embed + 'static + Send + Sync,
    {
        Self(PhantomData)
    }
}
impl<E> EmbeddedPagesProvider<E, false> {
    pub fn new_dynamic(_: E) -> Self
    where
        E: Embed + 'static + Send + Sync,
    {
        Self(PhantomData)
    }
}

impl<E, const STATIC: bool> EmbeddedPagesProvider<E, STATIC> {
    fn iter() -> impl IntoIterator<Item = (Cow<'static, [PathComponent]>, Cow<'static, str>)>
    where
        E: Embed,
    {
        E::iter().filter_map(|filename| {
            let path = filename
                .strip_suffix(".md")?
                .split_once(". ")?
                .0
                .split(['.', '\\', '/'])
                .map(|i| PathComponent::from_str_radix(i, 10))
                .try_collect()
                .ok()?;

            Some((Cow::Owned(path), filename))
        })
    }

    fn fetch(filename: Cow<'static, str>) -> Option<ManPageContent>
    where
        E: Embed,
    {
        let content = match E::get(&filename)?.data {
            Cow::Borrowed(d) => str::from_utf8(d).expect("Non utf8 man page").into(),
            Cow::Owned(d) => String::from_utf8(d).expect("Non utf8 man page").into(),
        };

        let title = match filename {
            Cow::Borrowed(filename) => get_title(filename).into(),
            Cow::Owned(filename) => get_title(&filename).to_owned().into(),
        };

        Some(ManPageContent { title, content })
    }
}

fn get_title(filename: &str) -> &str {
    filename
        .strip_suffix(".md")
        .unwrap()
        .split_once(". ")
        .unwrap()
        .1
        .trim()
}

impl<E, const STATIC: bool> Provider for EmbeddedPagesProvider<E, STATIC>
where
    E: Embed + 'static + Send + Sync,
{
    fn as_static(
        self,
    ) -> Result<
        impl IntoIterator<Item = (Cow<'static, [PathComponent]>, ManPageContent)>,
        Box<dyn DynamicProvider>,
    > {
        if !STATIC {
            let this = EmbeddedPagesProvider(self.0);
            return Err(Box::new(this));
        }
        Ok(Self::iter().into_iter().map(|(path, filename)| {
            let content = Self::fetch(filename).expect("In static mode files do not change");
            (path, content)
        }))
    }
}

impl<E> DynamicProvider for EmbeddedPagesProvider<E, false>
where
    E: Embed + 'static + Send + Sync,
{
    fn fetch<'s>(&'s self, path: &[PathComponent]) -> Option<Cow<'s, ManPageContent>> {
        Self::iter().into_iter().find_map(|(filepath, filename)| {
            (filepath == path)
                .then(|| Self::fetch(filename))
                // File deleted between iteration and fetching are considered
                // deleted
                .flatten()
                .map(Cow::Owned)
        })
    }

    fn descendants<'s, 'p, 'i>(
        &'s self,
        path: &'p [PathComponent],
    ) -> Box<dyn Iterator<Item = (Cow<'static, [PathComponent]>, Cow<'s, ManPageContent>)> + 'i>
    where
        's: 'i,
        'p: 'i,
    {
        Box::new(Self::iter().into_iter().filter_map(|(filepath, filename)| {
            (filepath.starts_with(path))
                .then(|| Self::fetch(filename))
                // File deleted between iteration and fetching are considered
                // deleted
                .flatten()
                .map(|f| (filepath.clone(), Cow::Owned(f)))
        }))
    }
}
