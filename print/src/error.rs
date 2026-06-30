use std::error::Error;

use pretty::{DocAllocator, DocBuilder, Pretty};

use crate::{Element, ErrorElement};

pub struct ErrorReport<'a>(pub &'a (dyn Error + 'a));

impl<'a> ErrorReport<'a> {
    pub fn new(error: &'a (dyn Error + 'a)) -> Self {
        Self(error)
    }
}

impl<'a, D> Pretty<'a, D, Element> for ErrorReport<'a>
where
    D: DocAllocator<'a, Element>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Element> {
        let mut doc = allocator.nil();

        let message = allocator
            .text(self.0.to_string())
            .annotate(Element::Error(Some(ErrorElement::Message)));
        doc = doc.append(message);
        doc = doc.append(allocator.hardline());

        let mut causes = Vec::new();
        let mut current = self.0.source();
        while let Some(cause) = current {
            causes.push(cause);
            current = cause.source();
        }

        if !causes.is_empty() {
            doc = doc.append(allocator.hardline());
            doc = doc.append(allocator.text("Caused by:"));

            for cause in causes {
                doc = doc.append(allocator.hardline());
                doc = doc.append(
                    allocator.text("  - ")
                        + allocator
                            .as_string(cause)
                            .annotate(Element::Error(Some(ErrorElement::Cause))),
                );
            }
        }

        doc.annotate(Element::Error(None))
    }
}
