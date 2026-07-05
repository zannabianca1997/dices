use pulldown_cmark::CowStr;

use crate::{DocAllocator, DocBuilder};

/// Rendered for code in the manual
pub trait CodeRender {
    /// What language this rendered handles
    fn handles(language: Option<&str>) -> bool;
    /// Render the code
    fn render<'a, D>(
        &self,
        allocator: &'a D,
        language: Option<&str>,
        tags: Option<&str>,
        code: CowStr<'a>,
    ) -> DocBuilder<'a, D>
    where
        D: DocAllocator<'a>,
        D::Doc: Clone;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultCodeRender;
impl CodeRender for DefaultCodeRender {
    fn handles(_: Option<&str>) -> bool {
        true
    }

    fn render<'a, D>(
        &self,
        allocator: &'a D,
        _: Option<&str>,
        _: Option<&str>,
        code: CowStr<'a>,
    ) -> DocBuilder<'a, D>
    where
        D: DocAllocator<'a>,
        D::Doc: Clone,
    {
        super::reflow_cowstr(allocator, code, true)
    }
}

impl<A, B> CodeRender for (A, B)
where
    A: CodeRender,
    B: CodeRender,
{
    fn handles(language: Option<&str>) -> bool {
        A::handles(language) || B::handles(language)
    }

    fn render<'a, D>(
        &self,
        allocator: &'a D,
        language: Option<&str>,
        tags: Option<&str>,
        code: CowStr<'a>,
    ) -> DocBuilder<'a, D>
    where
        D: DocAllocator<'a>,
        D::Doc: Clone,
    {
        if A::handles(language) {
            self.0.render(allocator, language, tags, code)
        } else {
            self.1.render(allocator, language, tags, code)
        }
    }
}

impl<R: CodeRender> CodeRender for &R {
    fn handles(language: Option<&str>) -> bool {
        R::handles(language)
    }

    fn render<'a, D>(
        &self,
        allocator: &'a D,
        language: Option<&str>,
        tags: Option<&str>,
        code: CowStr<'a>,
    ) -> DocBuilder<'a, D>
    where
        D: DocAllocator<'a>,
        D::Doc: Clone,
    {
        R::render(self, allocator, language, tags, code)
    }
}
