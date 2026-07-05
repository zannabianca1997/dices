//! Markdown printer
//!
//! Convert a stream of events from `pulldown_cmark` into an annotated document

use std::{borrow::Cow, iter::once, mem};

use dices_man::{Manual, PathComponent};
use itertools::Itertools;
use pulldown_cmark::{
    BrokenLink, BrokenLinkCallback, CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, Options,
    Parser, Tag, TagEnd,
};
use url::Url;

use crate::{DocAllocator, DocBuilder, Element, List, ListStyle, MarkdownElement, Pretty};
pub use code_rendered::{CodeRender, DefaultCodeRender};

mod code_rendered;

#[repr(transparent)]
pub struct Markdown<T: ?Sized> {
    pub text: T,
}

impl<T> Markdown<T> {
    pub fn new(text: T) -> Self
    where
        T: Sized,
    {
        Self { text }
    }
    pub fn new_ref(text: &T) -> &Self {
        unsafe {
            // Safety: #[repr(transparent)]
            &*(text as *const T as *const Self)
        }
    }
}

impl<'a, T> Markdown<&'a T>
where
    T: AsRef<str> + ?Sized,
{
    pub fn parser(text: &'a T) -> Parser<'a, impl BrokenLinkCallback<'a>> {
        Parser::new_with_broken_link_callback(
            text.as_ref(),
            Options::empty(),
            Some(broken_link_callback),
        )
    }
}

fn broken_link_callback<'a>(link: BrokenLink<'a>) -> Option<(CowStr<'a>, CowStr<'a>)> {
    if let Some((path, title)) = link.reference.split_once(". ")
        && let Ok(path) = path
            .split('.')
            .map(|s| PathComponent::from_str_radix(s, 10))
            .try_collect::<_, Vec<_>, _>()
        && let Some(page) = Manual::new().fetch(path)
    {
        if cfg!(debug_assertions) && page.title() != title.trim() {
            eprintln!(
                "Wrong page title in {link:?}: got {title}, expected {}",
                page.title()
            )
        }

        return Some((
            page.url().to_string().into(),
            match page.static_title() {
                Ok(t) => t.into(),
                Err(t) => t.to_owned().into(),
            },
        ));
    }

    if cfg!(debug_assertions) {
        eprintln!("Broken link: {link:?}")
    }
    None
}

#[derive(Debug, Clone)]
pub struct Ctx<R> {
    need_flow_separator: bool,
    code_render: R,
}

impl<R> Ctx<R> {
    pub fn new(code_render: R) -> Self {
        Self {
            need_flow_separator: false,
            code_render,
        }
    }
}

impl<R: Default> Default for Ctx<R> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<'a, D, R, T> Pretty<'a, D, Ctx<R>> for &'a Markdown<T>
where
    D: DocAllocator<'a>,
    T: AsRef<str> + ?Sized,
    D::Doc: Clone,
    R: CodeRender,
{
    fn pretty(self, allocator: &'a D, ctx: &mut Ctx<R>) -> DocBuilder<'a, D> {
        Markdown::new(&self.text).pretty(allocator, ctx)
    }
}
impl<'a, D, T, R> Pretty<'a, D, Ctx<R>> for Markdown<&'a T>
where
    D: DocAllocator<'a>,
    T: AsRef<str> + ?Sized,
    D::Doc: Clone,
    R: CodeRender,
{
    fn pretty(self, allocator: &'a D, ctx: &mut Ctx<R>) -> DocBuilder<'a, D> {
        Printer::new(&mut Self::parser(self.text))
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
        kind: CodeBlockKind<'a>,
    },
}
pub struct PrinterCtx<'r, 'a, R> {
    root: &'r mut Ctx<R>,
    stack: Vec<PrinterCtxFrame<'a>>,
    flow_state_stack: Vec<bool>,
}

impl<'r, 'a, R> PrinterCtx<'r, 'a, R> {
    pub fn new(root: &'r mut Ctx<R>) -> Self {
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
            Tag::CodeBlock(kind) => self.stack.push_mut(PrinterCtxFrame::CodeBlock { kind }),
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

impl<'e, 'a, D, Events, R> Pretty<'a, D, PrinterCtx<'e, 'a, R>> for Printer<'e, Events>
where
    Events: Iterator<Item = Event<'a>>,
    D: DocAllocator<'a>,
    D::Doc: Clone,
    R: CodeRender + 'e,
{
    fn pretty(self, allocator: &'a D, ctx: &mut PrinterCtx<'e, 'a, R>) -> DocBuilder<'a, D> {
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
                            allocator.text(number.to_string()).append(".")
                        } else {
                            allocator.text("-")
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
                        popped = line + marker + popped.indent(1);
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

                    let annotated = if let Some(element) = markdown_element(ctx, tag_end, frame) {
                        popped.annotate(Element::Markdown(Some(element)))
                    } else {
                        popped
                    };

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
                    if let PrinterCtxFrame::CodeBlock { kind } = ctx.current() {
                        let (language, tags) = match kind {
                            CodeBlockKind::Fenced(cow_str) => {
                                match cow_str.trim_start().split_once(char::is_whitespace) {
                                    Some((a, b)) => (Some(a.trim_end()), Some(b.trim())),
                                    None => (Some(cow_str.trim()).filter(|s| !s.is_empty()), None),
                                }
                            }
                            CodeBlockKind::Indented => (None, None),
                        };

                        *current += ctx
                            .root
                            .code_render
                            .render(allocator, language, tags, cow_str)
                    } else {
                        *current += reflow_cowstr(allocator, cow_str, false)
                    }
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

fn markdown_element<'e, 'a, R>(
    ctx: &mut PrinterCtx<'e, 'a, R>,
    tag_end: TagEnd,
    frame: PrinterCtxFrame<'_>,
) -> Option<MarkdownElement> {
    match tag_end {
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

        TagEnd::Link => {
            let PrinterCtxFrame::Generic {
                tag:
                    Tag::Link {
                        mut dest_url,
                        link_type,
                        ..
                    },
            } = frame
            else {
                unreachable!();
            };

            if link_type == LinkType::Email {
                dest_url = format!("mailto:{dest_url}").into();
            }

            match Url::parse(&dest_url) {
                Ok(url) => MarkdownElement::Link { url },
                Err(error) => {
                    if cfg!(debug_assertions) {
                        dbg!(dest_url.into_static(), error);
                    }
                    return None;
                }
            }
        }

        t @ (TagEnd::Image | TagEnd::HtmlBlock | TagEnd::BlockQuote(None)) => {
            todo!("Tag {t:?} still to implement")
        }

        t @ (TagEnd::BlockQuote(Some(_))
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
    }
    .into()
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
