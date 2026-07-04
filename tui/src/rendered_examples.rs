use std::{convert::Infallible, error::Error};

use dices_engine::{Engine, Evaluator, ui::Ui};
use dices_man::examples::Example;
use dices_parser::parse_scope_inner;
use dices_print::{
    DocAllocator, DocBuilder, Element, ErrorElement, Pretty as _, PromptElement,
    markdown::CodeRender, value,
};
use dices_std::{Std, StdOptions};
use dices_values::{Value, cast::push_down_if_injected, null::ValueNull, string::ValueString};
use pulldown_cmark::CowStr;

use crate::config::skin::Skin;

pub struct RenderedExamples<'a>(&'a Skin);

impl<'a> RenderedExamples<'a> {
    pub fn new(skin: &'a Skin) -> Self {
        Self(skin)
    }
}

// TODO: Implement printing in the examples
struct ExampleRenderUi;

impl Ui for ExampleRenderUi {
    type PrintError = Infallible;

    fn print(&self, _value: impl Into<Value>) -> Result<(), Self::PrintError> {
        todo!()
    }

    fn print_str<V: AsRef<str> + Into<ValueString>>(
        &self,
        _value: V,
    ) -> Result<(), Self::PrintError> {
        todo!()
    }

    fn print_md<V: AsRef<str> + Into<ValueString>>(
        &self,
        _value: V,
    ) -> Result<(), Self::PrintError> {
        todo!()
    }

    fn manual(&self, _item: &dices_man::ManPage) -> Result<(), Self::PrintError> {
        todo!()
    }
}

impl CodeRender for RenderedExamples<'_> {
    fn handles(language: Option<&str>) -> bool {
        matches!(language, None | Some("dices") | Some("dices-example"))
    }

    fn render<'a, D>(
        &self,
        allocator: &'a D,
        _language: Option<&str>,
        tags: Option<&str>,
        code: CowStr<'a>,
    ) -> DocBuilder<'a, D>
    where
        D: DocAllocator<'a>,
        D::Doc: Clone,
    {
        let skin = self.0;
        let mut engine = Engine::new([0u8; _], Std::new(StdOptions::sandboxed()));

        let example = Example::new(tags.unwrap_or(""), &code);

        let mut doc = allocator.nil();

        let prompt_char = if skin.graphical { "🎲" } else { ">>" };
        let indicator_char = if skin.graphical { "〉" } else { "> " };
        let multiline_char = "... ";

        let mut first = true;

        for cmd in &example.commands {
            let command_text = cmd.command();
            let parsed = match parse_scope_inner(&command_text.clone().into()) {
                Ok(p) => p,
                Err(e) => {
                    if cmd.hidden {
                        panic!("Error parsing hidden command: {e}");
                    }
                    if !first {
                        doc = doc.append(allocator.hardline());
                    }
                    first = false;
                    doc = doc.append(
                        allocator
                            .text(e.to_string())
                            .annotate(Element::Error(Some(ErrorElement::Message))),
                    );
                    continue;
                }
            };

            if cmd.hidden {
                match engine.eval(&parsed, ExampleRenderUi) {
                    Ok(_) => {}
                    Err(e) => panic!("Error in hidden command: {e}"),
                }
                continue;
            }

            if !first {
                doc = doc.append(allocator.hardline());
            }
            first = false;

            doc = doc.append(allocator.text(prompt_char).annotate(Element::Prompt(None)));
            doc = doc.append(
                allocator
                    .text(indicator_char)
                    .annotate(Element::Prompt(Some(PromptElement::Indicator))),
            );

            let mut lines = command_text.lines();
            if let Some(first_line) = lines.next() {
                doc = doc.append(allocator.text(first_line.to_owned()));
            }
            for line in lines {
                doc = doc.append(allocator.hardline());
                doc = doc.append(
                    allocator
                        .text(multiline_char)
                        .annotate(Element::Prompt(Some(PromptElement::Multiline))),
                );
                doc = doc.append(allocator.text(line.to_owned()));
            }

            doc = doc.append(allocator.hardline());

            match engine.eval(&parsed, ExampleRenderUi) {
                Ok(Value::Null(ValueNull)) => {}
                Ok(value) => {
                    let value = push_down_if_injected(value.clone()).unwrap_or(value);
                    let mut value_ctx = value::Ctx::default();
                    doc = doc.append(value.pretty(allocator, &mut value_ctx));
                }
                Err(error) => {
                    doc = doc.append(
                        allocator
                            .text(error.to_string())
                            .annotate(Element::Error(Some(ErrorElement::Message))),
                    );

                    let mut source = error.source();
                    while let Some(cause) = source {
                        doc = doc.append(allocator.hardline());
                        let cause_text = format!("Caused by: {}", cause);
                        doc = doc.append(
                            allocator
                                .text(cause_text)
                                .annotate(Element::Error(Some(ErrorElement::Cause))),
                        );
                        source = cause.source();
                    }
                }
            }
        }

        doc
    }
}
