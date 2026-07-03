use std::iter::{self, empty, once};

use dices_man::{ManPage, PathComponent};
use itertools::{Either, Itertools};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::{
    DocAllocator, DocBuilder, Element, Pretty,
    markdown::{CodeRender, Printer, PrinterCtx},
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
            .chain(Parser::new(self.content()));

        Printer::new(&mut doc)
            .pretty(allocator, &mut PrinterCtx::new(ctx))
            .annotate(Element::Markdown(None))
    }
}

/// Create the table of contents for the given page
///
/// Return empty if there are no nested pages
fn table_of_contents<'a>(page: &'a ManPage) -> impl IntoIterator<Item = Event<'a>> {
    let nested = page
        .descendants()
        // do not show the root
        .filter(|d| !d.path().is_empty())
        // might as well do it here
        // needed to print them in order
        .sorted()
        .collect_vec();

    if nested.len() <= 1 {
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

    let toc_body = nested
        .into_iter()
        .map(Some)
        .chain(once(None))
        // Scan the items
        //
        // Keeps track of the current path in the manual to build the correct
        // event sequence
        .scan(vec![], |current_path, page_or_end| {
            let mut events = vec![];
            // if inside an item or did already close it
            let mut flowing = true;

            let target_path = if let Some(page) = page_or_end.as_ref() {
                // Navigate to path
                page.path()
            } else {
                // Ending toc, close all items
                &[]
            };

            let page = if target_path.len() == current_path.len()
                && current_path
                    .last()
                    .is_some_and(|c| c + 1 == *target_path.last().unwrap())
            {
                // fast skip, just end this item and the counter
                // will go on accordingly

                *current_path.last_mut().unwrap() += 1;
                events.push(Event::End(TagEnd::Item));
                flowing = false;

                page_or_end.unwrap()
            } else {
                // full reroute

                while !target_path.starts_with(&current_path) {
                    if flowing {
                        events.push(Event::End(TagEnd::Item));
                    }

                    events.push(Event::End(TagEnd::List(true)));
                    flowing = true;
                    current_path.pop();
                }

                let Some(page) = page_or_end else {
                    // End of iteration
                    return Some(events);
                };

                while current_path != page.path() {
                    if !flowing {
                        events.push(Event::Start(Tag::Item));
                    }

                    events.push(Event::Start(Tag::List(Some(
                        page.path()[current_path.len()] as _,
                    ))));
                    flowing = false;
                    current_path.push(page.path()[current_path.len()]);
                }

                page
            };

            if !flowing {
                events.push(Event::Start(Tag::Item));
            }

            match page.static_title() {
                Ok(t) => events.extend(inline_md(t)),
                Err(t) => events.extend(inline_md(t).into_iter().map(|e| e.into_static())),
            }

            Some(events)
        })
        .flatten();

    Some(iter::chain(toc_title, toc_body)).into_iter().flatten()
}

fn inline_md<'a>(s: &'a str) -> impl IntoIterator<Item = Event<'a>> {
    Parser::new(s)
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
