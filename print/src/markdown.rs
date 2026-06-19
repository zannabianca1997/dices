use pretty::{DocAllocator, DocBuilder, Pretty};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::{Annotation, MarkdownElement};

pub struct Markdown<'text>(Parser<'text>);

impl<'text> Markdown<'text> {
    pub fn new(text: &'text str) -> Self {
        Self(Parser::new(text))
    }
}

fn tag_annotation(tag: &Tag<'_>) -> Option<Annotation> {
    match tag {
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
        TagEnd::Heading(_) | TagEnd::Strong | TagEnd::Emphasis
    )
}

impl<'a, D> Pretty<'a, D, Annotation> for Markdown<'a>
where
    D: DocAllocator<'a, Annotation>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Annotation> {
        let mut doc = allocator.nil();
        let mut annotations: Vec<Annotation> = Vec::new();

        for event in self.0 {
            match event {
                Event::Start(tag) => {
                    if let Some(ann) = tag_annotation(&tag) {
                        annotations.push(ann);
                    }
                }
                Event::End(tag_end) => {
                    if is_supported_end(&tag_end) {
                        annotations.pop();
                    }
                    if matches!(tag_end, TagEnd::Heading(_) | TagEnd::Paragraph) {
                        doc = doc.append(allocator.hardline());
                    }
                }
                Event::Text(text) => {
                    let mut text_doc = allocator.text(text.to_string());
                    for ann in annotations.iter().rev() {
                        text_doc = text_doc.annotate(*ann);
                    }
                    doc = doc.append(text_doc);
                }
                Event::Code(code) => {
                    let mut code_doc = allocator
                        .text(code.to_string())
                        .annotate(Annotation::Markdown(Some(MarkdownElement::InlineCode)));
                    for ann in annotations.iter().rev() {
                        code_doc = code_doc.annotate(*ann);
                    }
                    doc = doc.append(code_doc);
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
