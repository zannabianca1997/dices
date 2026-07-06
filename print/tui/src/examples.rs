//! Render `dices-example` code blocks found in the manual
//!
//! Each command in the example is echoed with a prompt and evaluated, its
//! result (or error) rendered below it.

use std::convert::Infallible;

use dices_engine::{Engine, Evaluator, ui::Ui};
use dices_man::examples::Example;
use dices_parser::parse_scope_inner;
use dices_print::{
    DocAllocator, DocBuilder, Element, Pretty as _, PromptElement, error::ErrorReport,
    markdown::CodeRender, value,
};
use dices_std::{Std, StdOptions};
use dices_values::{Value, cast::push_down_if_injected, null::ValueNull, string::ValueString};
use pretty::Pretty;
use pulldown_cmark::CowStr;

use crate::PromptDisplay;

pub struct TuiCodeRender<P>(P);

impl<P: PromptDisplay> TuiCodeRender<P> {
    pub fn new(prompt: P) -> Self {
        Self(prompt)
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

impl<P: PromptDisplay> CodeRender for TuiCodeRender<P> {
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
        let prompt = &self.0;
        let mut engine = Engine::new([0u8; _], Std::new(StdOptions::sandboxed()));

        let example = Example::new(tags.unwrap_or(""), &code);

        let mut commands = vec![];

        for cmd in &example.commands {
            let parsed = parse_scope_inner(&cmd.command().into());

            if !cmd.hidden {
                let mut prompt_doc = (allocator
                    .text(prompt.prompt_left())
                    .annotate(Element::Prompt(None))
                    + allocator
                        .text(prompt.prompt_indicator())
                        .annotate(Element::Prompt(Some(PromptElement::Indicator))))
                .annotate(Element::Prompt(None));

                let mut lines = cmd.command.iter().copied();
                if let Some(first_line) = lines.next() {
                    prompt_doc += allocator
                        .text(first_line.to_owned())
                        .annotate(Element::Ast(None));
                }
                for line in lines {
                    prompt_doc += allocator.hardline()
                        + allocator
                            .text(prompt.prompt_multiline_indicator())
                            .annotate(Element::Prompt(Some(PromptElement::Multiline)))
                            .annotate(Element::Prompt(None))
                        + allocator.text(line.to_owned()).annotate(Element::Ast(None));
                }
                commands.push(prompt_doc);
            }

            match parsed {
                Err(error) => commands.push(ErrorReport::new(&error).pretty(allocator)),
                Ok(parsed) => match engine.eval(&parsed, ExampleRenderUi) {
                    Ok(Value::Null(ValueNull)) => (),
                    Ok(value) => {
                        let value = push_down_if_injected(value.clone()).unwrap_or(value);
                        let mut value_ctx = value::Ctx::default();
                        commands.push(value.pretty(allocator, &mut value_ctx))
                    }
                    Err(error) => commands.push(ErrorReport::new(&error).pretty(allocator)),
                },
            };
        }

        allocator.intersperse(commands, allocator.hardline())
    }
}
