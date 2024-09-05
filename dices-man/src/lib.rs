//! This package contains  the manual pages for `dices`

#![feature(never_type)]
#![feature(box_patterns)]
#![feature(iter_intersperse)]
#![feature(mapped_lock_guards)]
#![feature(error_reporter)]

use std::{
    borrow::Cow,
    collections::HashMap,
    error::Report,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
    sync::{Mutex, MutexGuard, OnceLock},
};

use dices_ast::{
    intrisics::NoInjectedIntrisics,
    values::{Value, ValueNull},
};
use dices_engine::Engine;
use example::{CodeExample, CodeExampleCommand, CodeExamplePiece};
use itertools::Itertools;
use markdown::{
    mdast::{Code, Node},
    to_mdast, ParseOptions,
};
use pretty::DocAllocator;
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
    /// Width for the rendering
    width: usize,
}
impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            prompt: Cow::Borrowed(">>>"),
            prompt_cont: Cow::Borrowed("..."),
            seed: 0,
            width: 128,
        }
    }
}

/// Content of the cache for tha parsed markdown AST
struct AstCache {
    ast: Node,
    rendered: Mutex<HashMap<RenderOptions, Node>>,
}

/// A page of the manual
pub struct ManPage {
    /// The name of the page
    pub name: &'static str,
    /// The content of the page
    pub content: &'static str,
    /// The markdown ast of the page, if parsed
    ast: OnceLock<Box<AstCache>>,
}
impl ManPage {
    const fn new(name: &'static str, content: &'static str) -> Self {
        Self {
            name,
            content,
            ast: OnceLock::new(),
        }
    }

    fn ast_cache(&self) -> &AstCache {
        self.ast.get_or_init(|| {
            Box::new(AstCache {
                ast: to_mdast(&self.content, &ParseOptions::default()).unwrap(),
                rendered: Mutex::new(HashMap::new()),
            })
        })
    }

    /// The source ast, with the unrendered examples
    pub fn source(&self) -> &Node {
        &self.ast_cache().ast
    }

    /// The ast of the page, once the examples are rendered
    pub fn rendered(&self, options: RenderOptions) -> impl Deref<Target = Node> + '_ {
        let AstCache { ast, rendered } = self.ast_cache();
        // Lock the cache for ourselves
        // If poisoned, clear the cache and unpoison it.
        let rendered = rendered.lock().unwrap_or_else(|mut e| {
            **e.get_mut() = HashMap::new();
            rendered.clear_poison();
            e.into_inner()
        });
        // Get the cached value or render it
        MutexGuard::map(rendered, |rendered| {
            rendered
                .entry(options)
                .or_insert_with_key(|options| render_examples(ast.clone(), options))
        })
    }
}

fn render_examples(mut ast: Node, options: &RenderOptions) -> Node {
    // nodes that must be examined
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
        // parse it as an example
        let code: CodeExample = value.parse().expect(
            "The examples in the manual should be all well formatted, thanks to `dices-mantest`",
        );
        // initialize an engine, deterministic with regard of the seed and the code
        let mut engine: Engine<SmallRng, NoInjectedIntrisics> =
            Engine::new_with_rng(SmallRng::seed_from_u64({
                let mut hasher = DefaultHasher::new();
                options.seed.hash(&mut hasher);
                code.hash(&mut hasher);
                hasher.finish()
            }));
        // run all commands and concatenate the results
        let doc_arena = pretty::Arena::<()>::new();
        let res_arena = typed_arena::Arena::with_capacity(code.len());
        let doc = doc_arena.intersperse(
            (&*code).into_iter().filter_map(
                |CodeExamplePiece {
                     cmd:
                         CodeExampleCommand {
                             ignore,
                             command: box command,
                             src,
                         },
                     res: _,
                 }| {
                    let res = engine.eval_multiple(command);
                    if *ignore {
                        // only assert that the result is ok
                        if let Err(err) = res {
                            panic!("An example failed with {err}")
                        }
                        None
                    } else {
                        // print the command
                        let command =
                            doc_arena.intersperse(
                                src.lines().with_position().map(|(pos, line)| {
                                    doc_arena
                                        .text(match pos {
                                            itertools::Position::First
                                            | itertools::Position::Only => &*options.prompt,
                                            itertools::Position::Middle
                                            | itertools::Position::Last => &*options.prompt_cont,
                                        })
                                        .append(line)
                                }),
                                doc_arena.hardline(),
                            );
                        // move res into the arena
                        let res = &*res_arena.alloc(res);
                        // print the result or the error
                        let command_and_res = match res {
                            Ok(Value::Null(ValueNull)) => command,
                            Ok(res) => command.append(doc_arena.hardline()).append(res),
                            Err(err) => {
                                let report = Report::new(err).pretty(true);
                                command
                                    .append(doc_arena.hardline())
                                    .append(report.to_string())
                            }
                        };

                        Some(command_and_res)
                    }
                },
            ),
            doc_arena.hardline(),
        );
        // print the result
        value.clear();
        doc.render_fmt(options.width, value)
            .expect("Rendering should be infallible")
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
    /// Index of this directory
    Index,
    /// A directory of items
    Dir(ManDir),
}

pub static MANUAL: ManDir = include!(env!("MANUAL_RS"));
