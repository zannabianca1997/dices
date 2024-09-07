//! Test checking the well-formness of the manual

use markdown::mdast::{Link, Node};

use crate::{search, MANUAL};

/// The introduction must exist as it is shown when calling `help()`
#[test]
fn introduction_exist() {
    assert!(search("introduction").is_some())
}

/// The index must exist as it is shown when calling `help` with an invalid topic
#[test]
fn index_exist() {
    assert!(search("index").is_some())
}

/// Check that the links to manual page are all to existing manual pages
#[test]
fn manual_internal_links_are_not_dangling() {
    let mut dirs = vec![&MANUAL];
    while let Some(dir) = dirs.pop() {
        'read_dir: for &item in dir.content.values() {
            let page = match item {
                crate::ManItem::Page(page) => page,
                crate::ManItem::Index(_) => {
                    continue 'read_dir;
                }
                crate::ManItem::Dir(dir) => {
                    dirs.push(dir);
                    continue 'read_dir;
                }
            };
            // get the page ast
            let page = page.source();
            // visit the entire ast recursively
            let mut nodes = vec![page];
            'read_ast: while let Some(node) = nodes.pop() {
                // add the childrens to visit recursively
                nodes.extend(node.children().into_iter().flatten());
                // search only links
                let Node::Link(Link { url, .. }) = node else {
                    continue 'read_ast;
                };
                let Some(topic) = url.trim().strip_prefix("man:") else {
                    // not a link to the manual
                    continue 'read_ast;
                };
                // check the topic exist
                assert!(
                    search(topic).is_some(),
                    "The topic {topic} was referenced but does not exist"
                )
            }
        }
    }
}
