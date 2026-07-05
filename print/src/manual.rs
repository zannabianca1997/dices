use std::iter::{self, empty, once};

use dices_man::{ManPage, PathComponent};
use itertools::{Either, Itertools};
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};

use crate::{
    DocAllocator, DocBuilder, Element, Pretty,
    markdown::{CodeRender, Markdown, Printer, PrinterCtx},
};

pub type Ctx<R> = super::markdown::Ctx<R>;

impl<'a, D, R> Pretty<'a, D, Ctx<R>> for &'a ManPage
where
    D: DocAllocator<'a>,
    D::Doc: Clone,
    R: CodeRender,
{
    fn pretty(self, allocator: &'a D, ctx: &mut Ctx<R>) -> DocBuilder<'a, D> {
        let title = once(Event::Start(Tag::Heading {
            level: HeadingLevel::H1,
            id: None,
            classes: vec![],
            attrs: vec![],
        }))
        .chain(number(self.path()))
        .chain(inline_md(self.title()))
        .chain(once(Event::End(TagEnd::Heading(HeadingLevel::H1))));

        let index = table_of_contents(self);

        let mut doc = title
            .chain(index.into_iter())
            .chain(Markdown::parser(self.content()));

        Printer::new(&mut doc)
            .pretty(allocator, &mut PrinterCtx::new(ctx))
            .annotate(Element::Markdown(None))
    }
}

/// Create the table of contents for the given page
///
/// Return empty if there are no nested pages
fn table_of_contents<'a>(page: &'a ManPage) -> impl IntoIterator<Item = Event<'a>> {
    let children = page
        .children()
        // might as well do it here
        // needed to print them in order
        .sorted()
        .collect_vec();

    if children.is_empty() {
        return None.into_iter().flatten();
    }

    let toc_title = [
        Event::Start(Tag::Heading {
            level: HeadingLevel::H2,
            id: None,
            classes: vec![],
            attrs: vec![],
        }),
        Event::Text("Table of contents".into()),
        Event::End(TagEnd::Heading(HeadingLevel::H2)),
    ];

    let toc_body = children_list(children);

    Some(iter::chain(toc_title, toc_body)).into_iter().flatten()
}

fn children_list<'a>(
    children: impl IntoIterator<Item = ManPage>,
) -> impl IntoIterator<Item = Event<'a>> {
    let mut children = children.into_iter().peekable();
    let mut next_marker = *children.peek().unwrap().path().last().unwrap() as u64;
    once(Event::Start(Tag::List(Some(next_marker))))
        .chain(children.flat_map(move |page| {
            let marker = *page.path().last().unwrap() as u64;
            let opening = if marker != next_marker {
                Some([
                    Event::End(TagEnd::List(true)),
                    Event::Start(Tag::List(Some(marker))),
                ])
            } else {
                None
            };
            next_marker = marker + 1;

            let children = page.children().sorted().collect_vec();

            let children = (!children.is_empty())
                .then(move || children_list(children))
                .into_iter()
                .flatten();

            opening
                .into_iter()
                .flatten()
                .chain([
                    Event::Start(Tag::Item),
                    Event::Start(Tag::Link {
                        link_type: pulldown_cmark::LinkType::ReferenceUnknown,
                        dest_url: page.url().to_string().into(),
                        title: match page.static_title() {
                            Ok(t) => t.into(),
                            Err(t) => t.to_string().into(),
                        },
                        id: "".into(),
                    }),
                ])
                .chain(match page.static_title() {
                    Ok(t) => itertools::Either::Left(inline_md(t).into_iter()),
                    Err(t) => itertools::Either::Right(
                        inline_md(t)
                            .into_iter()
                            .map(Event::into_static)
                            .collect_vec()
                            .into_iter(),
                    ),
                })
                .chain(once(Event::End(TagEnd::Link)))
                .chain(children.collect_vec())
                .chain(once(Event::End(TagEnd::Item)))
        }))
        .chain(once(Event::End(TagEnd::List(true))))
}

fn inline_md<'a>(s: &'a str) -> impl IntoIterator<Item = Event<'a>> {
    Markdown::parser(s)
        .filter(|evt| evt != &Event::Start(Tag::Paragraph) && evt != &Event::End(TagEnd::Paragraph))
}

fn number<'a>(path: &[PathComponent]) -> impl IntoIterator<Item = Event<'a>> {
    if path.is_empty() {
        return Either::Right(empty());
    }
    Either::Left(
        [
            Event::Start(Tag::Strong),
            Event::Text(path.iter().format(".").to_string().into()),
            Event::Text(". ".into()),
            Event::End(TagEnd::Strong),
        ]
        .into_iter(),
    )
}
