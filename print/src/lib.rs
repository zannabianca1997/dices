#![doc = include_str!("../README.md")]

pub use element::*;

pub mod error;
pub mod manual;
pub mod markdown;
pub mod theme;
pub mod value;

pub mod element;

pub trait DocAllocator<'a>: pretty::DocAllocator<'a, Element> {}
impl<'a, T> DocAllocator<'a> for T where T: pretty::DocAllocator<'a, Element> {}

pub type DocBuilder<'a, D> = pretty::DocBuilder<'a, D, Element>;

pub trait Pretty<'a, D>
where
    D: DocAllocator<'a>,
{
    type Ctx;
    fn pretty(self, allocator: &'a D, ctx: &mut Self::Ctx) -> DocBuilder<'a, D>;

    fn with_ctx(self, ctx: Self::Ctx) -> WithContext<'a, D, Self>
    where
        Self: Sized,
    {
        WithContext { doc: self, ctx }
    }

    fn with_default_ctx(self) -> WithContext<'a, D, Self>
    where
        Self: Sized,
        Self::Ctx: Default,
    {
        Self::with_ctx(self, Default::default())
    }
}

pub struct WithContext<'a, D, T>
where
    T: Pretty<'a, D>,
    D: DocAllocator<'a>,
{
    ctx: T::Ctx,
    doc: T,
}

impl<'a, D, T> pretty::Pretty<'a, D, Element> for WithContext<'a, D, T>
where
    T: Pretty<'a, D>,
    D: DocAllocator<'a>,
{
    fn pretty(mut self, allocator: &'a D) -> DocBuilder<'a, D> {
        self.doc.pretty(allocator, &mut self.ctx)
    }
}
