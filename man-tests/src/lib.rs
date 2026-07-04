#![doc = include_str!("../README.md")]

use std::convert::Infallible;

use itertools::Itertools;
use snafu::{Report, ResultExt, Snafu};

use dices_engine::{Engine, EvalError, Evaluator, ui::Ui};
use dices_man::examples::{Command, Example};
use dices_parser::{
    ParseCommandError,
    matcher::{ParseMatcherError, parse_matcher},
    parse_scope_inner,
};
use dices_std::{Std, StdOptions};
use dices_values::{Value, string::ValueString};

#[cfg(test)]
mod tests {
    //! Tests extracted from the manual

    include!(env!("COLLECTED_TESTS"));
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(context(false))]
    ParseCommand {
        source: ParseCommandError,
    },
    #[snafu(context(false))]
    ParseMatcher {
        source: ParseMatcherError,
    },
    EvalErrorInExample {
        source: EvalError,
    },
    #[snafu(display("Expected {expected}, got {got}"))]
    ResultDidNotMatch {
        expected: String,
        got: Value,
    },
}

pub fn check_example_or_panic(example: &Example) {
    let Err(err) = check_example(example) else {
        // Example went well
        return;
    };

    panic!("{}", Report::from_error(err))
}

pub fn check_example(Example { tags, commands }: &Example) -> Result<(), Error> {
    if tags.contains(&"ignore") {
        return Ok(());
    }

    // Parse the commands
    let commands: Vec<_> = commands
        .into_iter()
        .map(
            |Command {
                 hidden: _,
                 command,
                 response,
             }| {
                let command = parse_scope_inner(&command.join("\n").into())?;
                let matcher = parse_matcher(&(*response).to_owned().into())?;

                Ok::<_, Error>((command, *response, matcher))
            },
        )
        .try_collect()?;

    if tags.contains(&"no_run") {
        return Ok(());
    }

    // Init the test engine
    let mut engine = Engine::new([0u8; _], Std::new(StdOptions::sandboxed()));

    for (command, expected, matcher) in commands {
        let response = engine
            .eval(&command, ExampleRunnerUi)
            .context(EvalErrorInExampleSnafu)?;

        if !matcher.matches(&response) {
            return Err(Error::ResultDidNotMatch {
                expected: expected.to_owned(),
                got: response,
            });
        }
    }

    Ok(())
}

// TODO: implement ui to check values against it
struct ExampleRunnerUi;

impl Ui for ExampleRunnerUi {
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
