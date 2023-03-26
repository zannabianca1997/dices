//! Build a generic help page with package description, list of command and authors.

use std::{cmp::Reverse, collections::HashMap, iter::once};

use termimad::minimad::{Text, TextTemplate};

use crate::cmd::{CmdDiscriminants, CMD_STRINGS};

fn get_help() -> Text<'static> {
    let text_template = TextTemplate::from(include_str!("general.md"));

    let mut expander = text_template.expander();
    expander
        .set("appname", env!("CARGO_PKG_NAME"))
        .set("appversion", env!("CARGO_PKG_VERSION"))
        .set("appdescription", env!("CARGO_PKG_DESCRIPTION"))
        .set("author", env!("CARGO_PKG_AUTHORS"));
    // start adding commands
    for (text, args, descr) in cmds_list() {
        expander
            .sub("cmds")
            .set("cmd-text", text.leak())
            .set("cmd-args", args)
            .set("cmd-descr", descr);
    }

    expander.expand()
}

fn cmds_list() -> impl Iterator<Item = (String, &'static str, &'static str)> {
    use CmdDiscriminants::*;
    let descrs = HashMap::from([
        (Throw, ("<EXPR>", "Execute a dice throw")),
        (Help, ("[CMD]", "Get help")),
        (Quit, ("", "Exit the program")),
    ]);
    let mut map = HashMap::new();
    for (text, cmd) in CMD_STRINGS.entries() {
        map.entry(cmd)
            .or_insert({
                let descr = descrs
                    .get(cmd)
                    .expect("`cmd_descr` should give descriptions of every command");
                (vec![], descr.0, descr.1)
            })
            .0
            .push(*text);
    }
    let mut entries: Vec<_> = map.into_iter().collect();
    entries.sort();
    entries.into_iter().map(|(cmd, (mut cmds, arg, descr))| {
        cmds.sort_unstable_by_key(|s| Reverse(s.len()));
        let cmds = cmds.into_iter().intersperse(" | ");
        let cmds: String = if *cmd == CmdDiscriminants::default() {
            once("[").chain(cmds).chain(once("]")).collect()
        } else {
            cmds.collect()
        };
        (cmds, arg, descr)
    })
}

lazy_static::lazy_static! {
    pub static ref HELP: Text<'static> = get_help();
}
