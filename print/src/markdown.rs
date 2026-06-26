use std::{marker::PhantomData, mem};

use pulldown_cmark::{CowStr, Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::{DocAllocator, DocBuilder, Element, List, ListStyle, MarkdownElement, Pretty};

#[repr(transparent)]
pub struct Markdown<T: ?Sized>(pub T);

impl<T: ?Sized> Markdown<T> {
    pub fn new(text: T) -> Self
    where
        T: Sized,
    {
        Self(text)
    }
    pub fn new_ref(text: &T) -> &Self {
        unsafe {
            // Safety: #[repr(transparent)]
            &*(text as *const T as *const Self)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ctx {}

impl Ctx {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Ctx {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, D, T> Pretty<'a, D> for &'a Markdown<T>
where
    T: ?Sized,
    D: DocAllocator<'a>,
    T: AsRef<str>,
    D::Doc: Clone,
{
    type Ctx = Ctx;

    fn pretty(self, allocator: &'a D, ctx: &mut Self::Ctx) -> DocBuilder<'a, D> {
        <Markdown<&'a T> as Pretty<'a, D>>::pretty(Markdown(&self.0), allocator, ctx)
    }
}
impl<'a, D, T> Pretty<'a, D> for Markdown<&'a T>
where
    D: DocAllocator<'a>,
    T: AsRef<str> + ?Sized,
    D::Doc: Clone,
{
    type Ctx = Ctx;

    fn pretty(self, allocator: &'a D, ctx: &mut Self::Ctx) -> DocBuilder<'a, D> {
        Printer::<_, FlowCtx>(&mut Parser::new(self.0.as_ref()), PhantomData)
            .pretty(
                allocator,
                &mut FlowCtx {
                    parent: ParentCtx::Root(ctx),
                },
            )
            .annotate(Element::Markdown(None))
    }
}

enum ParentCtx<'p> {
    Root(&'p Ctx),
    Flow(&'p FlowCtx<'p>),
    List(&'p ListCtx<'p>),
}

pub struct FlowCtx<'p> {
    parent: ParentCtx<'p>,
}

impl<'p> From<&'p Ctx> for FlowCtx<'p> {
    fn from(value: &'p Ctx) -> Self {
        Self {
            parent: ParentCtx::Root(value),
        }
    }
}

struct ListCtx<'p> {
    parent: ParentCtx<'p>,
    ordered: Option<u64>,
}

impl<'p> ListCtx<'p> {
    fn root(&self) -> &'p Ctx {
        let mut ancestor = &self.parent;
        loop {
            match *ancestor {
                ParentCtx::Root(ctx) => break ctx,
                ParentCtx::Flow(FlowCtx { parent, .. })
                | ParentCtx::List(ListCtx { parent, .. }) => ancestor = parent,
            }
        }
    }
}

pub struct Printer<'p, Events, Ctx>(&'p mut Events, PhantomData<Ctx>);

impl<'p, 'a, Events> Printer<'p, Events, FlowCtx<'a>> {
    pub fn new(events: &'p mut Events) -> Self
    where
        Events: Iterator<Item = Event<'a>>,
    {
        Self(events, PhantomData)
    }
}

impl<'p, 'a, 'c, Events, D> Pretty<'a, D> for Printer<'p, Events, FlowCtx<'c>>
where
    Events: Iterator<Item = Event<'a>>,
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    type Ctx = FlowCtx<'c>;

    fn pretty(self, allocator: &'a D, ctx: &mut Self::Ctx) -> DocBuilder<'a, D> {
        let mut stack = vec![];
        let mut doc = allocator.nil();
        let mut reflow = true;

        while let Some(event) = self.0.next() {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading { .. } => {
                        reflow = false;
                        stack.push(mem::replace(&mut doc, allocator.nil()))
                    }
                    Tag::List(ordered) => {
                        let mut ctx = ListCtx {
                            parent: ParentCtx::Flow(ctx),
                            ordered,
                        };
                        let parser = &mut *self.0;

                        doc += Printer::<_, ListCtx>(parser, PhantomData).pretty(allocator, &mut ctx)
                    }
                    Tag::Item => unreachable!("Emitted only inside a ListCtx"),
                    Tag::CodeBlock(_code_block_kind) => todo!(),
                    Tag::HtmlBlock | Tag::BlockQuote(_) | Tag::Link { .. } | Tag::Image { .. } => {
                        unimplemented!()
                    }
                    Tag::FootnoteDefinition(_)
                    | Tag::DefinitionList
                    | Tag::DefinitionListTitle
                    | Tag::DefinitionListDefinition
                    | Tag::Table(_)
                    | Tag::TableHead
                    | Tag::TableRow
                    | Tag::TableCell
                    | Tag::Strikethrough
                    | Tag::Superscript
                    | Tag::Subscript
                    | Tag::MetadataBlock(_) => unreachable!(),
                    Tag::Paragraph | Tag::Emphasis | Tag::Strong => {
                        stack.push(mem::replace(&mut doc, allocator.nil()))
                    }
                },
                Event::End(tag_end) => {
                    let Some(popped) = stack.pop() else {
                        // Popped out of this flow
                        break;
                    };
                    let markdown_element = match tag_end {
                        TagEnd::Paragraph => MarkdownElement::Paragraph,
                        TagEnd::Heading(heading_level) => {
                            reflow = true;
                            doc += allocator.hardline();
                            doc += allocator.hardline();

                            MarkdownElement::Header {
                                level: match heading_level {
                                    HeadingLevel::H1 => 1,
                                    HeadingLevel::H2 => 2,
                                    HeadingLevel::H3 => 3,
                                    HeadingLevel::H4 => 4,
                                    HeadingLevel::H5 => 4,
                                    HeadingLevel::H6 => 5,
                                },
                            }
                        }
                        TagEnd::Emphasis => MarkdownElement::Italic,
                        TagEnd::Strong => MarkdownElement::Bold,
                        _ => unreachable!(),
                    };
                    doc = popped.append(doc.annotate(Element::Markdown(Some(markdown_element))));
                }
                Event::Text(text) => doc += reflow_text(allocator, reflow, text),
                Event::Code(text) => {
                    doc += reflow_text(allocator, reflow, text)
                        .annotate(Element::Markdown(Some(MarkdownElement::InlineCode)))
                }
                Event::SoftBreak => {
                    doc += allocator.space();
                }
                Event::HardBreak => {
                    doc += allocator.hardline();
                    doc += allocator.hardline();
                }
                Event::Html(_) | Event::InlineHtml(_) | Event::Rule => unimplemented!(),
                Event::FootnoteReference(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::TaskListMarker(_) => unreachable!("Options for these events are disabled"),
            }
        }

        debug_assert!(stack.is_empty());
        doc
    }
}

fn reflow_text<'a, D>(allocator: &'a D, reflow: bool, text: CowStr<'a>) -> DocBuilder<'a, D>
where
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    if reflow {
        if let CowStr::Borrowed(text) = text {
            allocator.intersperse(text.split(char::is_whitespace), allocator.softline())
        } else {
            allocator.intersperse(
                text.split(char::is_whitespace).map(ToOwned::to_owned),
                allocator.softline(),
            )
        }
    } else {
        allocator.text(text)
    }
}
impl<'p, 'a, 'c, Events, D> Pretty<'a, D> for Printer<'p, Events, ListCtx<'c>>
where
    Events: Iterator<Item = Event<'a>>,
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    type Ctx = ListCtx<'c>;

    fn pretty(self, allocator: &'a D, ctx: &mut Self::Ctx) -> DocBuilder<'a, D> {
        let mut doc = allocator.nil();
        let style = match ctx.ordered.is_some() {
            true => ListStyle::Ordered,
            false => ListStyle::Unordered,
        };

        while let Some(event) = self.0.next() {
            match event {
                Event::Start(Tag::Item) => {
                    let marker = match ctx.ordered.as_mut() {
                        Some(count) => {
                            let txt = format!("{count}.");
                            *count += 1;
                            allocator.text(txt)
                        }
                        None => allocator.text("-"),
                    }
                    .annotate(Element::Markdown(Some(MarkdownElement::List {
                        style,
                        element: Some(List::Marker),
                    })));

                    let mut ctx = FlowCtx {
                        parent: ParentCtx::List(ctx),
                    };
                    let parser = &mut *self.0;

                    let content =
                        Printer::<_, FlowCtx>(parser, PhantomData).pretty(allocator, &mut ctx);

                    doc += marker
                        .append(content.indent(1))
                        .append(allocator.hardline())
                        .annotate(Element::Markdown(Some(MarkdownElement::List {
                            style,
                            element: Some(List::Item),
                        })))
                }
                Event::End(TagEnd::List(_)) => break,
                _ => unreachable!("List context contains only list items"),
            }
        }

        doc.annotate(Element::Markdown(Some(MarkdownElement::List {
            style,
            element: None,
        })))
    }
}
