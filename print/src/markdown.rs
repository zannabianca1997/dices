use std::{borrow::Cow, mem};

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
pub struct Ctx {
    need_flow_separator: bool,
}

impl Ctx {
    pub fn new(need_flow_separator: bool) -> Self {
        Self {
            need_flow_separator,
        }
    }
}

impl Default for Ctx {
    fn default() -> Self {
        Self::new(false)
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
        Printer::new(&mut Parser::new(self.0.as_ref()))
            .pretty(allocator, &mut PrinterCtx::new(ctx))
            .annotate(Element::Markdown(None))
    }
}

/// Printer context
#[derive(Debug, Default)]
enum PrinterCtxFrame<'a> {
    #[default]
    Root,
    Generic {
        tag: Tag<'a>,
    },
    List {
        ordered: Option<u64>,
    },
    ListItem {
        number: Option<u64>,
    },
}
pub struct PrinterCtx<'r, 'a> {
    root: &'r mut Ctx,
    stack: Vec<PrinterCtxFrame<'a>>,
    flow_state_stack: Vec<bool>,
}

impl<'r, 'a> PrinterCtx<'r, 'a> {
    pub fn new(root: &'r mut Ctx) -> Self {
        Self {
            root,
            stack: vec![],
            flow_state_stack: vec![],
        }
    }

    fn current(&self) -> &PrinterCtxFrame<'a> {
        self.stack.last().unwrap_or(&PrinterCtxFrame::Root)
    }

    fn current_mut(&mut self) -> Option<&mut PrinterCtxFrame<'a>> {
        self.stack.last_mut()
    }

    fn push(&mut self, tag: Tag<'a>) -> &mut PrinterCtxFrame<'a> {
        if let Tag::List(ordered) = tag {
            self.stack.push_mut(PrinterCtxFrame::List { ordered })
        } else if let Tag::Item = tag {
            let Some(PrinterCtxFrame::List { ordered, .. }) = self.current_mut() else {
                panic!("List item out of list");
            };
            let frame = PrinterCtxFrame::ListItem { number: *ordered };
            *ordered = ordered.map(|v| v + 1);
            self.stack.push_mut(frame)
        } else {
            self.stack.push_mut(PrinterCtxFrame::Generic { tag })
        }
    }

    fn pop(&mut self, tag_end: TagEnd) -> PrinterCtxFrame<'a> {
        let frame = self.stack.pop().expect("Unopened tag end");

        let tag_ended = match &frame {
            PrinterCtxFrame::Root => unreachable!(),
            PrinterCtxFrame::Generic { tag } => tag.to_end(),
            PrinterCtxFrame::List { ordered, .. } => TagEnd::List(ordered.is_some()),
            PrinterCtxFrame::ListItem { .. } => TagEnd::Item,
        };

        debug_assert_eq!(tag_end, tag_ended, "Mismatched end tags");

        frame
    }

    fn flow_state(&mut self) -> &mut bool {
        self.flow_state_stack
            .last_mut()
            .unwrap_or(&mut self.root.need_flow_separator)
    }

    fn open_flow_state_inner(&mut self) {
        self.flow_state_stack.push(false);
    }
    fn close_flow_state_inner(&mut self) {
        self.flow_state_stack.pop();
    }

    fn pop_flow_separator(&mut self) -> bool {
        mem::replace(self.flow_state(), false)
    }

    fn push_need_flow_separator(&mut self) {
        *self.flow_state() = true;
    }
}

pub struct Printer<'e, Events>(&'e mut Events);

impl<'e, 'a, Events> Printer<'e, Events> {
    pub fn new(events: &'e mut Events) -> Self
    where
        Events: Iterator<Item = Event<'a>>,
    {
        Self(events)
    }
}

impl<'e, 'a, D, Events> Pretty<'a, D> for Printer<'e, Events>
where
    Events: Iterator<Item = Event<'a>>,
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    type Ctx = PrinterCtx<'e, 'a>;

    fn pretty(self, allocator: &'a D, ctx: &mut Self::Ctx) -> DocBuilder<'a, D> {
        let mut docs = vec![allocator.nil()];

        for event in self.0 {
            let current = docs.last_mut().unwrap();
            match event {
                Event::Start(tag) => match ctx.push(tag) {
                    PrinterCtxFrame::ListItem { .. } => {
                        ctx.open_flow_state_inner();
                        docs.push(allocator.nil());
                    }
                    PrinterCtxFrame::Root => unreachable!(),
                    _ => docs.push(allocator.nil()),
                },
                Event::End(tag_end) => {
                    let frame = ctx.pop(tag_end);
                    let mut popped = docs.pop().unwrap();

                    if let PrinterCtxFrame::ListItem { number } = frame {
                        let marker = if let Some(number) = number {
                            allocator.text(number.to_string()).append(". ")
                        } else {
                            allocator.text("- ")
                        }
                        .annotate(Element::Markdown(Some(
                            MarkdownElement::List {
                                style: if number.is_some() {
                                    ListStyle::Ordered
                                } else {
                                    ListStyle::Unordered
                                },
                                element: Some(List::Marker),
                            },
                        )));

                        let line = allocator.column(|c| {
                            if c > 0 {
                                allocator.hardline()
                            } else {
                                allocator.nil()
                            }
                            .into_doc()
                        });

                        // Indent the list item content
                        popped = line + marker + popped.indent(0);
                    }

                    let is_flow = match tag_end {
                        TagEnd::Item => false,
                        TagEnd::Heading(_)
                        | TagEnd::List(_)
                        | TagEnd::Paragraph
                        | TagEnd::CodeBlock => true,
                        _ => false,
                    };
                    if tag_end == TagEnd::Item {
                        ctx.close_flow_state_inner();
                    }

                    let annotated = popped.annotate(Element::Markdown(Some(match tag_end {
                        TagEnd::Paragraph => MarkdownElement::Paragraph,
                        TagEnd::Heading(heading_level) => MarkdownElement::Header {
                            level: match heading_level {
                                HeadingLevel::H1 => 1,
                                HeadingLevel::H2 => 2,
                                HeadingLevel::H3 => 3,
                                HeadingLevel::H4 => 4,
                                HeadingLevel::H5 => 5,
                                HeadingLevel::H6 => 6,
                            },
                        },

                        TagEnd::List(ordered) => MarkdownElement::List {
                            style: if ordered {
                                ListStyle::Ordered
                            } else {
                                ListStyle::Unordered
                            },
                            element: None,
                        },
                        TagEnd::Item => MarkdownElement::List {
                            style: {
                                let PrinterCtxFrame::List { ordered, .. } = ctx.current() else {
                                    unreachable!()
                                };
                                if ordered.is_some() {
                                    ListStyle::Ordered
                                } else {
                                    ListStyle::Unordered
                                }
                            },
                            element: Some(List::Item),
                        },

                        TagEnd::Emphasis => MarkdownElement::Italic,
                        TagEnd::Strong => MarkdownElement::Bold,

                        t @ (TagEnd::Link
                        | TagEnd::Image
                        | TagEnd::HtmlBlock
                        | TagEnd::CodeBlock) => todo!("Tag {t:?} still to implement"),

                        t @ (TagEnd::BlockQuote(_)
                        | TagEnd::FootnoteDefinition
                        | TagEnd::DefinitionList
                        | TagEnd::DefinitionListTitle
                        | TagEnd::DefinitionListDefinition
                        | TagEnd::Table
                        | TagEnd::TableHead
                        | TagEnd::TableRow
                        | TagEnd::TableCell
                        | TagEnd::Strikethrough
                        | TagEnd::Superscript
                        | TagEnd::Subscript
                        | TagEnd::MetadataBlock(_)) => {
                            unimplemented!("Tag {t:?} not supported (not emitted without options)")
                        }
                    })));

                    let current = docs.last_mut().unwrap();

                    if is_flow && ctx.pop_flow_separator() {
                        *current += allocator.hardline() + allocator.hardline();
                    }

                    *current += annotated;

                    if is_flow {
                        ctx.push_need_flow_separator();
                    }
                }
                Event::Text(cow_str) => *current += reflow_cowstr(allocator, cow_str),
                Event::Code(cow_str) => {
                    *current += reflow_cowstr(allocator, cow_str)
                        .annotate(Element::Markdown(Some(MarkdownElement::InlineCode)))
                }
                Event::SoftBreak => *current += allocator.space(),
                Event::HardBreak => *current += allocator.hardline(),
                e @ (Event::Html(_) | Event::InlineHtml(_) | Event::Rule) => {
                    todo!("Event {e:?} still to implement")
                }
                e @ (Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::FootnoteReference(_)
                | Event::TaskListMarker(_)) => {
                    unimplemented!("Event {e:?} not supported (not emitted without options)")
                }
            }
        }

        debug_assert_eq!(docs.len(), 1, "Unclosed tags");
        docs.into_iter().next().unwrap()
    }
}

fn reflow_cowstr<'a, D>(allocator: &'a D, s: CowStr<'a>) -> DocBuilder<'a, D>
where
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    match Cow::<'a, str>::from(s) {
        Cow::Owned(s) => allocator.intersperse(
            s.split(char::is_whitespace).map(ToOwned::to_owned),
            allocator.softline(),
        ),
        Cow::Borrowed(s) => allocator.reflow(s),
    }
}
