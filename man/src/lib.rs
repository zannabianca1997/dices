//! This package contains  the manual pages for `dices`

use std::{
    borrow::Cow,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
    sync::{Arc, LazyLock, OnceLock},
};

use dices_ast::{
    intrisics::NoInjectedIntrisics,
    value::{Value, ValueNull},
};
use dices_engine::Engine;
use example::{CodeExample, CodeExampleCommand, CodeExamplePiece};
use itertools::Itertools;
use markdown::{
    mdast::{self, Code, Node},
    to_mdast, ParseOptions,
};
use pretty::DocAllocator;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use report::Report;

pub mod example;
mod report;

/// Options to render the examples in the manual pages
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderOptions {
    /// The prompt for the command: `>>> `
    pub prompt: Cow<'static, str>,
    /// The continue prompt for longer command: `... `
    pub prompt_cont: Cow<'static, str>,
    /// The seed for the example rng
    pub seed: u64,
    /// Width for the rendering
    pub width: usize,
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

/// Parse options used to parse the manual pages markdown
#[must_use]
pub const fn man_parse_options() -> ParseOptions {
    mdast2minimad::md_parse_options()
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct RenderKey {
    content: &'static str,
    options: RenderOptions,
}

/// Rendered manual pages cache
static RENDERS: LazyLock<mini_moka::sync::Cache<RenderKey, Arc<Node>>> =
    LazyLock::new(|| mini_moka::sync::Cache::builder().build());

/// A page of the manual
pub struct ManPage {
    /// The name of the page
    pub name: &'static str,
    /// The content of the page
    pub content: &'static str,
    /// The markdown ast of the page, if parsed
    ast: OnceLock<Node>,
}
impl ManPage {
    const fn new(name: &'static str, content: &'static str) -> Self {
        Self {
            name,
            content,
            ast: OnceLock::new(),
        }
    }

    /// The source ast, with the unrendered examples
    pub fn source(&self) -> &Node {
        self.ast
            .get_or_init(|| to_mdast(self.content, &man_parse_options()).unwrap())
    }

    /// The ast of the page, once the examples are rendered
    pub fn rendered(&self, options: RenderOptions) -> Arc<Node> {
        let key = RenderKey {
            content: self.content,
            options,
        };
        if let Some(cached) = RENDERS.get(&key) {
            return cached;
        }
        let render = Arc::new(render_examples(self.source().clone(), &key.options));
        RENDERS.insert(key, render.clone());
        render
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
        if lang.as_ref().is_none_or(|l| l != "dices") {
            // do not examine code that is not a `dices` code
            continue;
        }
        // parse it as an example
        let code: CodeExample = value.parse().expect(
            "The examples in the manual should be all well formatted, thanks to `dices-mantest`",
        );
        // initialize an engine, deterministic with regard of the seed and the code
        let mut engine: Engine<Xoshiro256PlusPlus, NoInjectedIntrisics, _> =
            Engine::new_with_rng(SeedableRng::seed_from_u64({
                let mut hasher = DefaultHasher::new();
                options.seed.hash(&mut hasher);
                code.hash(&mut hasher);
                hasher.finish()
            }));
        // run all commands and concatenate the results
        let doc_arena = pretty::Arena::<()>::new();
        let res_arena = typed_arena::Arena::with_capacity(code.len());
        let doc = doc_arena.intersperse(
            code.iter().filter_map(
                |CodeExamplePiece {
                     cmd:
                         CodeExampleCommand {
                             ignore,
                             command,
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
                                let report = Report::new(err);
                                command.append(doc_arena.hardline()).append(&report)
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
            .expect("Rendering should be infallible");
    }
    ast
}

/// A subdirectory of the manual
pub struct ManDir {
    /// The name of the subdirectory
    pub name: &'static str,
    /// The content of the subdirectory
    pub content: phf::OrderedMap<&'static str, &'static ManItem>,
    /// The index of the subdirectory, if rendered
    index: OnceLock<Node>,
}

impl ManDir {
    const fn new(
        name: &'static str,
        content: phf::OrderedMap<&'static str, &'static ManItem>,
    ) -> Self {
        Self {
            name,
            content,
            index: OnceLock::new(),
        }
    }
}

/// The index of a manual dir
pub struct ManIndex;
impl ManIndex {
    const fn new() -> Self {
        Self
    }
}

/// A item of the manual
pub enum ManItem {
    /// A single page
    Page(ManPage),
    /// Index of this directory
    Index(ManIndex),
    /// A directory of items
    Dir(ManDir),
}

/// Possible contents of a manual topic
#[derive(Clone, Copy)]
pub enum ManTopicContent {
    /// A manual page
    Page(&'static ManPage),
    /// Index of a directory
    Index(&'static ManDir),
}
impl ManTopicContent {
    /// The ast of the topic
    #[must_use]
    pub fn rendered<'r>(&self, options: RenderOptions) -> impl Deref<Target = Node> + 'r {
        enum RenderedRef<P: Deref<Target = Node>, I: Deref<Target = Node>> {
            Page(P),
            Index(I),
        }
        impl<P: Deref<Target = Node>, I: Deref<Target = Node>> Deref for RenderedRef<P, I> {
            type Target = Node;

            fn deref(&self) -> &Self::Target {
                match self {
                    RenderedRef::Page(p) => p,
                    RenderedRef::Index(i) => i,
                }
            }
        }
        match self {
            ManTopicContent::Page(p) => RenderedRef::Page(p.rendered(options)),
            ManTopicContent::Index(dir) => {
                RenderedRef::Index(dir.index.get_or_init(|| render_index(dir)))
            }
        }
    }
    #[must_use]
    pub const fn is_page(&self) -> bool {
        matches!(self, Self::Page(_))
    }
}

/// Create the index of a page
fn render_index(dir: &ManDir) -> Node {
    use markdown::mdast::*;

    fn list_item(name: &str, key: &str) -> Paragraph {
        // parse the name as markdown
        let mut children = markdown_one_line(name);
        children.extend([
            Node::Text(Text {
                value: " (".to_owned(),
                position: None,
            }),
            Node::InlineCode(InlineCode {
                value: key.to_owned(),
                position: None,
            }),
            Node::Text(Text {
                value: ")".to_owned(),
                position: None,
            }),
        ]);
        Paragraph {
            children,
            position: None,
        }
    }

    fn list_of(dir: &ManDir) -> List {
        List {
            children: dir
                .content
                .entries()
                .map(|(&key, &v)| {
                    Node::ListItem(ListItem {
                        children: match v {
                            ManItem::Page(p) => vec![Node::Paragraph(list_item(p.name, key))],
                            ManItem::Index(_) => vec![Node::Paragraph(list_item("Index", "index"))],
                            ManItem::Dir(d) => vec![
                                Node::Paragraph(list_item(d.name, key)),
                                Node::List(list_of(d)),
                            ],
                        },
                        position: None,
                        spread: false,
                        checked: None,
                    })
                })
                .collect(),
            position: None,
            ordered: false,
            start: None,
            spread: false,
        }
    }

    Node::Root(Root {
        children: vec![
            Node::Heading(Heading {
                children: {
                    let mut children = vec![Node::Text(Text {
                        value: "Index of \"".to_owned(),
                        position: None,
                    })];
                    children.append(&mut markdown_one_line(dir.name));
                    children.push(Node::Text(Text {
                        value: "\"".to_owned(),
                        position: None,
                    }));
                    children
                },
                position: None,
                depth: 1,
            }),
            Node::List(list_of(dir)),
        ],
        position: None,
    })
}

fn markdown_one_line(name: &str) -> Vec<Node> {
    let Node::Root(mdast::Root {
        mut children,
        position: _,
    }) = markdown::to_mdast(name, &ParseOptions::gfm()).expect("Markdown has no syntax error")
    else {
        panic!("`to_mdast` should always emit a `Root` node")
    };
    if children.len() > 1 {
        panic!("The name should contain a single paragrah")
    }
    let Node::Paragraph(mdast::Paragraph {
        children,
        position: _,
    }) = children.pop().unwrap_or(Node::Paragraph(mdast::Paragraph {
        children: vec![],
        position: None,
    }))
    else {
        panic!("The name should contain a paragraph")
    };
    children
}

/// Lookup a specific topic
#[must_use]
pub fn search(topic: &str) -> Option<ManTopicContent> {
    let mut topic = topic.split('/');
    let name = topic.next_back()?;

    let mut dir = &MANUAL;
    for part in topic {
        if let ManItem::Dir(child) = dir.content.get(part)? {
            dir = child;
        } else {
            return None;
        }
    }
    Some(match dir.content.get(name)? {
        ManItem::Page(page) => ManTopicContent::Page(page),
        ManItem::Index(_) => ManTopicContent::Index(dir),
        ManItem::Dir(dir) => ManTopicContent::Index(dir),
    })
}
#[must_use]
pub fn index() -> ManTopicContent {
    search("index").unwrap()
}

pub static MANUAL: ManDir = include!(env!("MANUAL_RS"));

#[cfg(test)]
mod tests;

/// Check if the std library is fully documented
#[cfg(any(feature = "test_std_handle", test))]
pub fn std_library_is_represented<InjectedIntrisic: dices_ast::intrisics::InjectedIntr>() {
    assert!(
        search("std").is_some(),
        "The std library is fully undocumented!"
    );
    let mut paths = vec![(
        "std".to_owned(),
        dices_engine::dices_std::<InjectedIntrisic>(),
    )];
    while let Some((path, map)) = paths.pop() {
        // iter throught the map
        for (name, value) in map {
            let path = path.clone() + "/" + &*name;
            // check it is documented
            let Some(topic) = search(&path) else {
                panic!("The topic {path} is missing");
            };
            // do not recurse if a page is expaining the whole map
            if topic.is_page() {
                continue;
            }
            // if a map, recurse
            if let Value::Map(map) = value {
                paths.push((path, map));
            }
        }
    }
}
