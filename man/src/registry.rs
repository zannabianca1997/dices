use std::{
    borrow::Cow,
    marker::PhantomData,
    sync::{LazyLock, RwLock, RwLockReadGuard},
};

use elsa::sync::FrozenVec;
use itertools::Itertools;

use crate::{ManPage, ManPageContent, PathComponent};

pub mod embedded;
pub mod linked;

/// A provider for manual pages
pub trait Provider {
    /// Try to solve the provider as static pages
    fn as_static(
        self,
    ) -> Result<
        impl IntoIterator<Item = (Cow<'static, [PathComponent]>, ManPageContent)>,
        Box<dyn DynamicProvider>,
    >;
}

/// A provider whose pages can change
pub trait DynamicProvider: Send + Sync + 'static {
    /// Get the given page if this provider has it
    fn fetch<'s>(&'s self, path: &[PathComponent]) -> Option<Cow<'s, ManPageContent>>;
    /// Get all pages in this provider with a path starting with the given one
    fn descendants<'s, 'p, 'i>(
        &'s self,
        path: &'p [PathComponent],
    ) -> Box<dyn Iterator<Item = (Cow<'static, [PathComponent]>, Cow<'s, ManPageContent>)> + 'i>
    where
        's: 'i,
        'p: 'i;
}

impl Provider for Box<dyn DynamicProvider> {
    fn as_static(
        self,
    ) -> Result<
        impl IntoIterator<Item = (Cow<'static, [PathComponent]>, ManPageContent)>,
        Box<dyn DynamicProvider>,
    > {
        Err::<[_; 0], _>(self)
    }
}

static REGISTRY: LazyLock<Shared> = LazyLock::new(Shared::new);

struct Shared {
    /// Cached entries sorted by path
    cached: RwLock<Vec<Entry>>,
    /// Non cacheable providers, sorted by registration order
    providers: FrozenVec<Box<dyn DynamicProvider>>,
}

impl Shared {
    fn new() -> Self {
        let this = Self {
            cached: RwLock::new(Vec::new()),
            providers: FrozenVec::new(),
        };

        // Register pages collected via the linker
        this.register(&linked::LINKED_PAGES);

        // Register pages from the base manual
        this.register(embedded::EmbeddedPagesProvider::new(
            embedded::EmbeddedPages,
        ));

        this
    }

    fn register<P: Provider>(&self, p: P) {
        let pages = match p.as_static() {
            Ok(pages) => pages,
            Err(dynamic) => {
                self.providers.push(dynamic);
                return;
            }
        };
        let mut cached = self.cached.write().unwrap();

        // Registers all pages, keeping the array sorted
        for (path, page) in pages {
            match cached.binary_search_by_key(&&*path, |e| e.path) {
                Err(pos) => {
                    cached.insert(pos, Entry::new(path, page));
                }
                Ok(pos) => panic!(
                    "Page collision for {}: `{}` overwrites `{}`",
                    path.iter().format("."),
                    page.title(),
                    cached[pos].title
                ),
            }
        }
    }
}

/// Entry in the register
struct Entry {
    path: &'static [PathComponent],
    title: &'static str,
    content: &'static str,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(other.path)
    }
}

impl Entry {
    fn new(
        path: Cow<'static, [PathComponent]>,
        ManPageContent { title, content }: ManPageContent,
    ) -> Self {
        fn leak_slice<T: Clone>(t: Cow<'static, [T]>) -> &'static [T] {
            match t {
                Cow::Borrowed(t) => t,
                Cow::Owned(t) => t.leak(),
            }
        }
        fn leak_str(t: Cow<'static, str>) -> &'static str {
            match t {
                Cow::Borrowed(t) => t,
                Cow::Owned(t) => t.leak(),
            }
        }
        Self {
            path: leak_slice(path),
            title: leak_str(title),
            content: leak_str(content),
        }
    }

    /// Create the page for this entry
    fn materialize_page(&self, manual: Manual) -> ManPage {
        ManPage {
            manual,
            path: self.path.into(),
            content: ManPageContent {
                title: self.title.into(),
                content: self.content.into(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Manual {
    /// Only one global registry, we do not need to actually store the pointer
    _priv: PhantomData<&'static Shared>,
}

impl Manual {
    /// Obtain an handle to the manual
    pub fn new() -> Self {
        // Ensure initialization
        LazyLock::force(&REGISTRY);
        // Now the user can copy this all times they want
        Self { _priv: PhantomData }
    }

    fn registry(&self) -> &'static Shared {
        let registry = LazyLock::get(&REGISTRY);
        unsafe {
            // Safety: we have &self, so we know that `new` has been called
            // _somewhere_ this unsafe can be trivially written in safe by
            // changing _priv to an actual pointer.
            registry.unwrap_unchecked()
        }
    }

    /// Get the first page of the manual
    pub fn first(&self) -> ManPage {
        self.descendants(&[])
            .sorted()
            .next()
            .expect("Static pages should guarantee a non empty manual")
    }

    /// Find a manual page
    pub fn fetch<'p, P>(&self, path: P) -> Option<ManPage>
    where
        P: AsRef<[PathComponent]> + Into<Cow<'static, [PathComponent]>>,
    {
        let registry = self.registry();

        // Fetch from cached static pages
        let cached = registry.cached.read().unwrap();
        if let Ok(pos) = cached.binary_search_by_key(&path.as_ref(), |e| e.path) {
            return Some(cached[pos].materialize_page(*self));
        }
        drop(cached);

        // Check dynamic providers
        for provider in registry.providers.iter() {
            if let Some(page) = provider.fetch(path.as_ref()) {
                return Some(self.make_page(path, page));
            }
        }

        // All providers failed
        None
    }

    fn make_page<P>(&self, path: P, page: Cow<'_, ManPageContent>) -> ManPage
    where
        P: Into<Cow<'static, [PathComponent]>>,
    {
        ManPage {
            manual: *self,
            path: path.into(),
            content: page.into_owned(),
        }
    }

    /// Find a manual page
    pub(crate) fn descendants<'p, P>(&self, path: &'p P) -> Descendant<'p, P>
    where
        Descendant<'p, P>: IntoIterator<Item = ManPage>,
    {
        let registry = self.registry();
        Descendant {
            manual: *self,
            path,
            cached: Some((0, registry.cached.read().unwrap())),
            current: None,
            providers: registry.providers.iter(),
        }
    }

    /// Add a new page or set of pages to the global manual
    ///
    /// The easiest way of using it is using the impl of Provider on (Path, Content):
    /// ```
    /// # use dices_man::{Manual, ManPageContent};
    /// # let manual = Manual::new();
    /// manual.register((&[99,99,3], "Test Registered Page", "Hello World!"));
    /// assert_eq!(manual.fetch(&[99,99,3]).unwrap().title(), "Test Registered Page");
    /// ```
    pub fn register<P: Provider>(&self, p: P) {
        self.registry().register(p);
    }
}

pub struct Descendant<'p, P> {
    manual: Manual,
    path: &'p P,
    cached: Option<(usize, RwLockReadGuard<'p, Vec<Entry>>)>,
    current: Option<
        Box<
            dyn Iterator<Item = (Cow<'static, [PathComponent]>, Cow<'static, ManPageContent>)> + 'p,
        >,
    >,
    providers: elsa::sync::Iter<'static, Box<dyn DynamicProvider>>,
}

impl<'p, P> Descendant<'p, P> {
    pub fn is_holding_lock(&self) -> bool {
        self.cached.is_some()
    }
}

impl<'p, P> Iterator for Descendant<'p, P>
where
    P: AsRef<[PathComponent]>,
{
    type Item = ManPage;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((pos, guard)) = self.cached.as_mut() {
            while let Some(entry) = guard.get(*pos) {
                *pos += 1;

                if entry.path.starts_with(self.path.as_ref()) {
                    return Some(entry.materialize_page(self.manual));
                }
            }
            // Drop the guard and free the lock
            self.cached = None;
        }

        if self.current.is_none() {
            self.current = Some(self.providers.next()?.descendants(self.path.as_ref()))
        }

        if let Some((path, page)) = self.current.as_mut().and_then(Iterator::next) {
            return Some(self.manual.make_page(path, page));
        }

        None
    }
}

/// Root manual page
#[linked::distributed_slice(linked::LINKED_PAGES)]
static ROOT_PAGE: linked::LinkedPage = linked::LinkedPage {
    path: &[],
    title: "Index",
    content: "",
};

// ==  Provider helpers
impl<P> Provider for (P, ManPageContent)
where
    P: Into<Cow<'static, [PathComponent]>>,
{
    fn as_static(
        self,
    ) -> Result<
        impl IntoIterator<Item = (Cow<'static, [PathComponent]>, ManPageContent)>,
        Box<dyn DynamicProvider>,
    > {
        Ok([(self.0.into(), self.1)])
    }
}

impl<P, T, C> Provider for (P, T, C)
where
    P: Into<Cow<'static, [PathComponent]>>,
    T: Into<Cow<'static, str>>,
    C: Into<Cow<'static, str>>,
{
    fn as_static(
        self,
    ) -> Result<
        impl IntoIterator<Item = (Cow<'static, [PathComponent]>, ManPageContent)>,
        Box<dyn DynamicProvider>,
    > {
        (self.0, ManPageContent::new(self.1, self.2)).as_static()
    }
}
