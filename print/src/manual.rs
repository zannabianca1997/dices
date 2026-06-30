use std::iter::{empty, once};

use dices_man::{ManPage, PathComponent};
use itertools::{Either, Itertools};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::{
    DocAllocator, DocBuilder, Pretty,
    markdown::{Printer, PrinterCtx},
};

pub type Ctx = super::markdown::Ctx;

impl<'a, D> Pretty<'a, D> for &'a ManPage
where
    D: DocAllocator<'a>,
    D::Doc: Clone,
{
    type Ctx = Ctx;

    fn pretty(self, allocator: &'a D, ctx: &mut Self::Ctx) -> DocBuilder<'a, D> {
        let title = once(Event::Start(Tag::Heading {
            level: HeadingLevel::H1,
            id: None,
            classes: vec![],
            attrs: vec![],
        }))
        .chain(number(self.path()))
        .chain(inline_md(self.title()))
        .chain(once(Event::End(TagEnd::Heading(HeadingLevel::H1))));

        let nested = self
            .descendants()
            .filter(|d| !d.path().is_empty())
            .sorted()
            .collect_vec();

        let index = if !nested.is_empty() {
            Either::Left(
                [
                    Event::Start(Tag::Heading {
                        level: HeadingLevel::H2,
                        id: None,
                        classes: vec![],
                        attrs: vec![],
                    }),
                    Event::Text("Content table".into()),
                    Event::End(TagEnd::Heading(HeadingLevel::H2)),
                ]
                .into_iter()
                .chain(
                    nested
                        .into_iter()
                        .map(Some)
                        .chain(once(None))
                        .scan((true, vec![]), |(flowing, current_path), page_or_end| {
                            let mut events = vec![];

                            let target_path = if let Some(page) = page_or_end.as_ref() {
                                page.path()
                            } else {
                                &[]
                            };

                            let page = if *flowing
                                && target_path.len() == current_path.len()
                                && current_path
                                    .last()
                                    .is_some_and(|c| c + 1 == *target_path.last().unwrap())
                            {
                                // fast skip, just end this item and the counter
                                // will go on accordingly

                                *current_path.last_mut().unwrap() += 1;
                                events.push(Event::End(TagEnd::Item));
                                *flowing = false;

                                page_or_end.unwrap()
                            } else {
                                // full reroute

                                while !target_path.starts_with(&current_path) {
                                    if *flowing {
                                        events.push(Event::End(TagEnd::Item));
                                        *flowing = false;
                                    }

                                    events.push(Event::End(TagEnd::List(true)));
                                    *flowing = true;
                                    current_path.pop();
                                }

                                let Some(page) = page_or_end else {
                                    // End of iteration
                                    return Some(events);
                                };

                                while current_path != page.path() {
                                    if !*flowing {
                                        events.push(Event::Start(Tag::Item));
                                        *flowing = true;
                                    }

                                    events.push(Event::Start(Tag::List(Some(
                                        page.path()[current_path.len()] as _,
                                    ))));
                                    *flowing = false;
                                    current_path.push(page.path()[current_path.len()]);
                                }

                                page
                            };

                            if !*flowing {
                                events.push(Event::Start(Tag::Item));
                                *flowing = true;
                            }

                            match page.static_title() {
                                Ok(t) => events.extend(inline_md(t)),
                                Err(t) => {
                                    events.extend(inline_md(t).into_iter().map(|e| e.into_static()))
                                }
                            }

                            Some(events)
                        })
                        .flatten(),
                ),
            )
        } else {
            Either::Right([])
        };

        let mut doc = title
            .chain(index.into_iter())
            .chain(Parser::new(self.content()));

        Printer::new(&mut doc).pretty(allocator, &mut PrinterCtx::new(ctx))
    }
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
