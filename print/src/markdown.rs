//! Markdown printer
//!
//! Convert a stream of events from `pulldown_cmark` into an annotated document

use std::{borrow::Cow, iter::once, mem};

use itertools::Itertools;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Parser, Tag, TagEnd};

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
    pub fn new() -> Self {
        Self {
            need_flow_separator: false,
        }
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
    CodeBlock {
        _kind: CodeBlockKind<'a>,
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
        match tag {
            Tag::List(ordered) => self.stack.push_mut(PrinterCtxFrame::List { ordered }),
            Tag::Item => {
                let Some(PrinterCtxFrame::List { ordered, .. }) = self.current_mut() else {
                    panic!("List item out of list");
                };
                let frame = PrinterCtxFrame::ListItem { number: *ordered };
                *ordered = ordered.map(|v| v + 1);
                self.stack.push_mut(frame)
            }
            Tag::CodeBlock(kind) => self
                .stack
                .push_mut(PrinterCtxFrame::CodeBlock { _kind: kind }),
            _ => self.stack.push_mut(PrinterCtxFrame::Generic { tag }),
        }
    }

    fn pop(&mut self, tag_end: TagEnd) -> PrinterCtxFrame<'a> {
        let frame = self.stack.pop().expect("Unopened tag end");

        let tag_ended = match &frame {
            PrinterCtxFrame::Root => unreachable!(),
            PrinterCtxFrame::Generic { tag } => tag.to_end(),
            PrinterCtxFrame::List { ordered, .. } => TagEnd::List(ordered.is_some()),
            PrinterCtxFrame::ListItem { .. } => TagEnd::Item,
            PrinterCtxFrame::CodeBlock { .. } => TagEnd::CodeBlock,
        };

        debug_assert_eq!(tag_end, tag_ended, "Mismatched end tags");

        frame
    }

    fn flow_state(&mut self) -> &mut bool {
        self.flow_state_stack
            .last_mut()
            .unwrap_or(&mut self.root.need_flow_separator)
    }

    /// Open a inner flow state (like inside a complex list item)
    fn open_flow_state_inner(&mut self) {
        self.flow_state_stack.push(false);
    }

    /// Close a inner flow state
    fn close_flow_state_inner(&mut self) {
        self.flow_state_stack.pop();
    }

    /// Remove the flow separator flag
    fn pop_flow_separator(&mut self) -> bool {
        mem::replace(self.flow_state(), false)
    }

    /// Set the flow separator flag
    ///
    /// This flag marks that a flow element has been printed, and a empty line
    /// need to be emitted before the next flow element
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

                    let is_flow = matches!(
                        tag_end,
                        TagEnd::Heading(_)
                            | TagEnd::List(_)
                            | TagEnd::Paragraph
                            | TagEnd::CodeBlock
                    );

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

                        TagEnd::CodeBlock => MarkdownElement::Code { inline: false },

                        t @ (TagEnd::Link | TagEnd::Image | TagEnd::HtmlBlock) => {
                            todo!("Tag {t:?} still to implement")
                        }

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
                Event::Text(cow_str) => {
                    *current += reflow_cowstr(
                        allocator,
                        cow_str,
                        matches!(ctx.current(), PrinterCtxFrame::CodeBlock { .. }),
                    )
                }
                Event::Code(cow_str) => {
                    *current += reflow_cowstr(allocator, cow_str, true).annotate(Element::Markdown(
                        Some(MarkdownElement::Code { inline: true }),
                    ))
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

fn reflow_cowstr<'a, D>(allocator: &'a D, s: CowStr<'a>, preserve_spaces: bool) -> DocBuilder<'a, D>
where
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    if preserve_spaces {
        match Cow::<'a, str>::from(s) {
            Cow::Owned(s) => {
                merge_with_hardlines(allocator, separate_spaces(&s).map(StrOrSpace::into_static))
            }
            Cow::Borrowed(s) => merge_with_hardlines(allocator, separate_spaces(&s)),
        }
    } else {
        let s = Cow::<'a, str>::from(s);
        let before = s
            .starts_with(char::is_whitespace)
            .then(|| allocator.softline());
        let after = s
            .ends_with(char::is_whitespace)
            .then(|| allocator.softline());
        match s {
            Cow::Owned(s) => allocator.intersperse(
                s.split_whitespace().map(ToOwned::to_owned),
                allocator.softline(),
            ),
            Cow::Borrowed(s) => allocator.intersperse(s.split_whitespace(), allocator.softline()),
        }
        .enclose(before, after)
    }
}

enum StrOrSpace<'a> {
    Str(&'a str),
    String(String),
    Space(char),
}

impl StrOrSpace<'_> {
    fn into_static<'a>(self) -> StrOrSpace<'a> {
        match self {
            StrOrSpace::Str(s) => StrOrSpace::String(s.to_owned()),
            StrOrSpace::String(s) => StrOrSpace::String(s),
            StrOrSpace::Space(s) => StrOrSpace::Space(s),
        }
    }
}

impl<'a, D, A: 'a> pretty::Pretty<'a, D, A> for StrOrSpace<'a>
where
    D: pretty::DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        match self {
            StrOrSpace::Str(s) => allocator.text(s),
            StrOrSpace::String(s) => allocator.text(s),
            StrOrSpace::Space(s) => allocator.as_string(s),
        }
    }
}

fn separate_spaces<'a>(src: &'a str) -> impl Iterator<Item = StrOrSpace<'a>> {
    src.char_indices()
        .filter_map(|(pos, ch)| ch.is_whitespace().then_some((pos, Some(ch))))
        .chain(once((src.len(), None)))
        .scan(0, |start, (end, space)| {
            let str = (*start != end).then_some(StrOrSpace::Str(&src[*start..end]));
            *start = end + space.map(|ch| ch.len_utf8()).unwrap_or_default();

            Some([str, space.map(StrOrSpace::Space)])
        })
        .flatten()
        .flatten()
}

fn merge_with_hardlines<'a, D>(
    allocator: &'a D,
    elements: impl Iterator<Item = StrOrSpace<'a>>,
) -> DocBuilder<'a, D>
where
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    let batches = elements.peekable().batching(|f| {
        if f.peek().is_none() {
            return None;
        }
        let elements = f.take_while(|e| !matches!(e, StrOrSpace::Space('\n')));
        Some(allocator.intersperse(elements, allocator.softline_()))
    });
    let with_hardlines = Itertools::intersperse(batches, allocator.hardline());
    with_hardlines
        .reduce(|a, b| a.append(b))
        .unwrap_or(allocator.nil())
}
