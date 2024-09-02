//! This package contains  the manual pages for `dices`

#![feature(never_type)]
#![feature(box_patterns)]
#![feature(iter_intersperse)]
#![feature(mapped_lock_guards)]
#![feature(error_reporter)]

use std::{
    borrow::Cow,
    collections::HashMap,
    error::{Error, Report},
    fmt::Write,
    hash::{DefaultHasher, Hash, Hasher},
    mem,
    ops::Deref,
    sync::{Mutex, MutexGuard, OnceLock},
};

use dices_ast::values::{Value, ValueNull};
use dices_engine::{Engine, SolveError};
use example::{CodeExample, CodeExampleCommand, CodeExamplePiece};
use markdown::{
    mdast::{Code, Node},
    to_mdast, ParseOptions,
};
use rand::{rngs::SmallRng, SeedableRng};

pub mod example;

/// Options to render the examples in the manual pages
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderOptions {
    /// The prompt for the command: `>>> `
    prompt: Cow<'static, str>,
    /// The continue prompt for longer command: `... `
    prompt_cont: Cow<'static, str>,
    /// The seed for the example rng
    seed: u64,
}
impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            prompt: Cow::Borrowed(">>>"),
            prompt_cont: Cow::Borrowed("..."),
            seed: 0,
        }
    }
}

/// A page of the manual
pub struct ManPage {
    /// The name of the page
    pub name: &'static str,
    /// The content of the page
    pub content: &'static str,
    /// The markdown ast of the page, if parsed
    ast: OnceLock<(Node, Mutex<HashMap<RenderOptions, Node>>)>,
}
impl ManPage {
    const fn new(name: &'static str, content: &'static str) -> Self {
        Self {
            name,
            content,
            ast: OnceLock::new(),
        }
    }

    fn ast_storage(&self) -> &(Node, Mutex<HashMap<RenderOptions, Node>>) {
        self.ast.get_or_init(|| {
            (
                to_mdast(&self.content, &ParseOptions::default()).unwrap(),
                Mutex::new(HashMap::new()),
            )
        })
    }

    /// The source ast, with the unrendered examples
    pub fn source(&self) -> &Node {
        &self.ast_storage().0
    }

    /// The ast of the page, once the examples are rendered
    pub fn rendered(&self, options: RenderOptions) -> impl Deref<Target = Node> + '_ {
        let (ast, cache) = self.ast_storage();
        // Lock the cache for ourselves
        // If poisoned, clear the cache and unpoison it.
        let cache = cache.lock().unwrap_or_else(|mut e| {
            **e.get_mut() = HashMap::new();
            cache.clear_poison();
            e.into_inner()
        });
        // Get the cached value or render it
        MutexGuard::map(cache, |cache| {
            cache
                .entry(options)
                .or_insert_with_key(|options| render_examples(ast.clone(), options))
        })
    }
}

fn render_examples(mut ast: Node, options: &RenderOptions) -> Node {
    let mut nodes = vec![&mut ast];
    while let Some(node) = nodes.pop() {
        let Node::Code(Code {
            value,
            position: _,
            lang,
            meta: _meta,
        }) = node
        else {
            // recover all the childrens
            nodes.extend(node.children_mut().into_iter().flatten());
            continue;
        };
        if !lang.as_ref().is_some_and(|l| l == "dices") {
            // do not examine code that is not a `dices` code
            continue;
        }
        // steal the original code
        let src = mem::take(value);
        // parse it as an example
        let code: CodeExample = src.parse().expect(
            "The examples in the manual should be all well formatted, thanks to `dices-mantest`",
        );
        // initialize an engine, deterministic with regard of the seed and the code
        let mut engine = Engine::new_with_rng(SmallRng::seed_from_u64({
            let mut hasher = DefaultHasher::new();
            options.seed.hash(&mut hasher);
            code.hash(&mut hasher);
            hasher.finish()
        }));
        // run all commands
        for CodeExamplePiece {
            cmd:
                CodeExampleCommand {
                    ignore,
                    command: box command,
                    src,
                },
            res: _,
        } in &*code
        {
            let res = engine.eval_multiple(command);
            if *ignore {
                // only assert that the result is ok
                if let Err(err) = res {
                    panic!("An example failed with {err}")
                }
            } else {
                // print the command
                value.push_str(&options.prompt);
                for val in src
                    .lines()
                    .intersperse(&format!("\n{}", options.prompt_cont))
                {
                    value.push_str(val)
                }
                value.push('\n');

                // print the result or the error
                match res {
                    Ok(Value::Null(ValueNull)) => (),
                    Ok(res) => writeln!(value, "{res}").unwrap(),
                    Err(err) => {
                        let report = Report::new(err).pretty(true);
                        writeln!(value, "{report}").unwrap()
                    }
                }
            }
        }
        // remove eccessive newlines
        while value.ends_with(['\n', '\r']) {
            value.pop();
        }
    }
    ast
}

/// A subdirectory of the manual
pub struct ManDir {
    /// The name of the subdirectory
    pub name: &'static str,
    /// The content of the subdirectory
    pub content: phf::OrderedMap<&'static str, &'static ManItem>,
}

/// A item of the manual
pub enum ManItem {
    /// A single page
    Page(ManPage),
    /// A directory of items
    Dir(ManDir),
}

pub static MANUAL: ManDir = include!(env!("MANUAL_RS"));
