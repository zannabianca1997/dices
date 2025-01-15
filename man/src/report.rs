use std::{error::Error, iter};

use pretty::Pretty;

pub(super) struct Report<E>(E);

impl<E> Report<E> {
    pub(super) fn new(error: E) -> Self {
        Self(error)
    }
}

impl<'a, D, A, E: Error> Pretty<'a, D, A> for &Report<E>
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A> + 'a,
    pretty::DocBuilder<'a, D, A>: Clone,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        let err = &self.0;
        let main_msg = allocator
            .text("Error:")
            .append(allocator.space())
            .append(allocator.text(err.to_string()));
        let mut source = err.source();
        if source.is_none() {
            return main_msg;
        };
        main_msg
            .append(allocator.hardline())
            .append(allocator.hardline())
            .append("Caused by:")
            .append(
                allocator
                    .intersperse(
                        iter::from_fn(|| {
                            if let Some(current) = source {
                                source = current.source();
                                Some(
                                    allocator
                                        .text("-")
                                        .append(allocator.space())
                                        .append(allocator.text(current.to_string())),
                                )
                            } else {
                                None
                            }
                        }),
                        allocator.hardline(),
                    )
                    .hang(2),
            )
    }
}
