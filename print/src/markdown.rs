use std::borrow::Cow;

use itertools::Itertools;
use pretty::{DocAllocator, DocBuilder};
use pulldown_cmark::{Event, Parser, Tag};

use crate::{Element, List, ListStyle, MarkdownElement, Pretty};

pub struct Markdown<'a>(pub Cow<'a, str>);

/// State of a list currently being rendered.
struct ListCtx {
    ordered: Option<u64>,
}

impl ListCtx {
    /// The styling kind of this list.
    fn style(&self) -> ListStyle {
        if self.ordered.is_some() {
            ListStyle::Ordered
        } else {
            ListStyle::Unordered
        }
    }
}

fn tag_element(tag: &Tag<'_>, parent_ctx: Option<&mut Ctx>) -> Option<Element> {
    match tag {
        Tag::Paragraph => Some(Element::Markdown(None)),
        Tag::Heading { level, .. } => Some(Element::Markdown(Some(MarkdownElement::Header {
            level: *level as u8,
        }))),
        Tag::Strong => Some(Element::Markdown(Some(MarkdownElement::Bold))),
        Tag::Emphasis => Some(Element::Markdown(Some(MarkdownElement::Italic))),
        Tag::List(ordered) => Some(Element::Markdown(Some(MarkdownElement::List {
            style: if ordered.is_some() {
                ListStyle::Ordered
            } else {
                ListStyle::Unordered
            },
            element: None,
        }))),
        Tag::Item if let Some(Ctx::List(ListCtx { ordered })) = parent_ctx => {
            Some(Element::Markdown(Some(MarkdownElement::List {
                style: if let Some(ordered) = ordered {
                    *ordered += 1;
                    ListStyle::Ordered
                } else {
                    ListStyle::Unordered
                },
                element: Some(List::Item),
            })))
        }
        Tag::Item => unreachable!("Items are always in a list"),
        Tag::BlockQuote(_)
        | Tag::CodeBlock(_)
        | Tag::HtmlBlock
        | Tag::Link { .. }
        | Tag::Image { .. } => unimplemented!(),
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
    }
}
fn tag_ctx(tag: &Tag<'_>) -> Option<Ctx> {
    match tag {
        Tag::List(ordered) => Some(Ctx::List(ListCtx { ordered: *ordered })),
        _ => None,
    }
}

struct RenderStackFrame<'a, D: DocAllocator<'a, Element> + 'a> {
    tag: Tag<'a>,
    doc: DocBuilder<'a, D, Element>,
    ctx: Option<Ctx>,
}
enum Ctx {
    List(ListCtx),
}
impl<'a, D: DocAllocator<'a, Element> + 'a> RenderStackFrame<'a, D> {
    pub fn finalize(self, parents: &mut [Self]) -> DocBuilder<'a, D, Element> {
        let parent_ctx = parents.iter_mut().rev().find_map(|p| p.ctx.as_mut());
        if let Some(element) = tag_element(&self.tag, parent_ctx) {
            self.doc.annotate(element)
        } else {
            self.doc
        }
    }
}

impl<'a, D> Pretty<'a, D> for &'a Markdown<'a>
where
    D: DocAllocator<'a, Element> + 'a,
    DocBuilder<'a, D, Element>: Clone,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Element> {
        let mut docs = nunny::vec![RenderStackFrame {
            tag: Tag::Paragraph,
            doc: allocator.nil(),
            ctx: None,
        }];

        for event in Parser::new(self.0.as_ref()) {
            match event {
                Event::Start(tag) => docs.push(RenderStackFrame {
                    ctx: tag_ctx(&tag),
                    tag,
                    doc: allocator.nil(),
                }),
                Event::End(tag_end) => {
                    assert!(docs.len() > 1);
                    let frame = unsafe { docs.as_mut_vec().pop().unwrap() };
                    debug_assert_eq!(tag_end, frame.tag.to_end());
                    let doc = frame.finalize(&mut docs);
                    docs.last_mut().doc += doc;
                }
                Event::Text(cow_str) | Event::Html(cow_str) => {
                    docs.last_mut().doc += allocator.text(cow_str)
                }
                Event::Code(cow_str) => {
                    docs.last_mut().doc += allocator
                        .text(cow_str)
                        .annotate(Element::Markdown(Some(MarkdownElement::InlineCode)))
                }
                Event::SoftBreak => docs.last_mut().doc += allocator.space(),
                Event::HardBreak => {
                    docs.last_mut().doc += allocator.hardline() + allocator.hardline()
                }
                Event::InlineHtml(_) | Event::Rule => unimplemented!(),
                Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::FootnoteReference(_)
                | Event::TaskListMarker(_) => unreachable!(),
            }
        }

        let Ok(root) = docs.into_iter().exactly_one() else {
            unreachable!()
        };
        root.finalize(&mut [])
    }
}
