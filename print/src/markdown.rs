use pretty::{DocAllocator, DocBuilder, Pretty};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::{Annotation, List, ListStyle, MarkdownElement};

pub struct Markdown<T>(pub T);

/// State of a list currently being rendered.
struct ListCtx {
    /// `Some(n)` holds the number of the next item for an ordered list;
    /// `None` marks an unordered (bullet) list.
    ordered: Option<u64>,
    /// Width of the marker emitted for the current item, used to indent any
    /// nested list underneath it.
    marker_width: usize,
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

fn tag_annotation(tag: &Tag<'_>) -> Option<Annotation> {
    match tag {
        Tag::Paragraph => Some(Annotation::Markdown(None)),
        Tag::Heading { level, .. } => Some(Annotation::Markdown(Some(MarkdownElement::Header {
            level: *level as u8,
        }))),
        Tag::Strong => Some(Annotation::Markdown(Some(MarkdownElement::Bold))),
        Tag::Emphasis => Some(Annotation::Markdown(Some(MarkdownElement::Italic))),
        _ => None,
    }
}

fn is_supported_end(tag_end: &TagEnd) -> bool {
    matches!(
        tag_end,
        TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::Strong | TagEnd::Emphasis
    )
}

impl<'a, D, T> Pretty<'a, D, Annotation> for Markdown<T>
where
    D: DocAllocator<'a, Annotation> + 'a,
    DocBuilder<'a, D, Annotation>: Clone,
    T: AsRef<str>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Annotation> {
        let mut doc = allocator.nil();
        let mut annotations: Vec<Annotation> = Vec::new();
        let mut lists: Vec<ListCtx> = Vec::new();
        // Whether any content has been appended yet, so the first list item does
        // not start with a spurious leading blank line.
        let mut emitted = false;

        for event in Parser::new(self.0.as_ref()) {
            match event {
                Event::Start(Tag::List(start)) => {
                    let ctx = ListCtx {
                        ordered: start,
                        marker_width: 0,
                    };
                    annotations.push(Annotation::List {
                        style: ctx.style(),
                        element: None,
                    });
                    lists.push(ctx);
                }
                Event::Start(Tag::Item) => {
                    // Indent under the markers of any enclosing list items.
                    let indent: usize = lists[..lists.len() - 1]
                        .iter()
                        .map(|ctx| ctx.marker_width)
                        .sum();

                    if emitted {
                        doc = doc.append(allocator.hardline());
                    }

                    let ctx = lists.last_mut().expect("an item is always inside a list");
                    let style = ctx.style();
                    let marker = match ctx.ordered {
                        Some(n) => {
                            ctx.ordered = Some(n + 1);
                            format!("{n}. ")
                        }
                        None => "- ".to_string(),
                    };
                    ctx.marker_width = marker.len();

                    annotations.push(Annotation::List {
                        style,
                        element: Some(List::Item),
                    });

                    // The marker glyph carries its own annotation (innermost, so
                    // it wins) wrapped by the active list/item annotations.
                    let mut marker_doc = allocator.text(marker).annotate(Annotation::List {
                        style,
                        element: Some(List::Marker),
                    });
                    for ann in annotations.iter().rev() {
                        marker_doc = marker_doc.annotate(*ann);
                    }
                    doc = doc.append(marker_doc.indent(indent));
                    emitted = true;
                }
                Event::Start(tag) => {
                    if let Some(ann) = tag_annotation(&tag) {
                        annotations.push(ann);
                    }
                }
                Event::End(tag_end) => {
                    if is_supported_end(&tag_end) {
                        annotations.pop();
                    }
                    match tag_end {
                        TagEnd::List(_) => {
                            annotations.pop(); // the `List { element: None, .. }`
                            lists.pop();
                            // Separate the finished list from following content.
                            if lists.is_empty() {
                                doc = doc.append(allocator.hardline());
                            }
                        }
                        TagEnd::Item => {
                            annotations.pop(); // the `List { element: Some(Item), .. }`
                        }
                        // Paragraphs inside list items would otherwise add a
                        // blank line between tight items; only break outside.
                        TagEnd::Heading(_) | TagEnd::Paragraph if lists.is_empty() => {
                            doc = doc.append(allocator.hardline());
                        }
                        _ => {}
                    }
                }
                Event::Text(text) => {
                    let mut text_doc = allocator.text(text.to_string());
                    for ann in annotations.iter().rev() {
                        text_doc = text_doc.annotate(*ann);
                    }
                    doc = doc.append(text_doc);
                    emitted = true;
                }
                Event::Code(code) => {
                    let mut code_doc = allocator
                        .text(code.to_string())
                        .annotate(Annotation::Markdown(Some(MarkdownElement::InlineCode)));
                    for ann in annotations.iter().rev() {
                        code_doc = code_doc.annotate(*ann);
                    }
                    doc = doc.append(code_doc);
                    emitted = true;
                }
                Event::SoftBreak => {
                    doc = doc.append(allocator.text(" "));
                }
                Event::HardBreak => {
                    doc = doc
                        .append(allocator.hardline())
                        .append(allocator.hardline());
                }
                _ => {}
            }
        }

        doc
    }
}
