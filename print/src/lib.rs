#![doc = include_str!("../README.md")]

use std::marker::PhantomData;

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

pub trait Pretty<'a, D, Ctx> {
    fn pretty(self, allocator: &'a D, ctx: &mut Ctx) -> DocBuilder<'a, D>
    where
        D: DocAllocator<'a>;

    fn with_ctx(self, ctx: Ctx) -> WithContext<Self, D, Ctx>
    where
        Self: Sized,
    {
        WithContext {
            doc: self,
            allocator: PhantomData,
            ctx,
        }
    }

    fn with_default_ctx(self) -> WithContext<Self, D, Ctx>
    where
        Self: Sized,
        Ctx: Default,
    {
        Self::with_ctx(self, Default::default())
    }
}

pub struct WithContext<T, D, C> {
    ctx: C,
    allocator: PhantomData<D>,
    doc: T,
}

impl<'a, D, T, C> pretty::Pretty<'a, D, Element> for WithContext<T, D, C>
where
    T: Pretty<'a, D, C>,
    D: DocAllocator<'a>,
{
    fn pretty(mut self, allocator: &'a D) -> DocBuilder<'a, D> {
        self.doc.pretty(allocator, &mut self.ctx)
    }
}
