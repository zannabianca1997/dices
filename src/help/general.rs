//! Build a generic help page with package description, list of command and authors.

use std::{cmp::Reverse, collections::HashMap, fmt::Write, iter::once};

use pad::PadStr;

use crate::cmd::{CmdDiscriminants, CmdStrings};

fn cmd_descr() -> HashMap<CmdDiscriminants, (&'static str, &'static str)> {
    use CmdDiscriminants::*;
    HashMap::from([
        (Throw, ("<EXPR>", "Execute a dice throw")),
        (Help, ("[CMD]", "Get help")),
        (Quit, ("", "Exit the program")),
    ])
}

lazy_static::lazy_static! {
    pub static ref HELP: &'static str = {
        let descr = env!("CARGO_PKG_DESCRIPTION");
        let authors = env!("CARGO_PKG_AUTHORS");
        let cmd_list = {
            let data : Vec<_> = {
                let descrs = cmd_descr();
                let mut map = HashMap::new();
                for (text, cmd) in CmdStrings.entries() {
                    map.entry(cmd).or_insert({
                        let descr = descrs.get(cmd).expect("`cmd_descr` should give descriptions of every command");
                        (vec![], descr.0, descr.1)
                    }).0.push(*text);
                }
                let mut entries: Vec<_> = map.into_iter().collect();
                entries.sort();
                entries.into_iter().map(|(cmd, (mut cmds, arg, descr))|{
                    cmds.sort_unstable_by_key(|s| Reverse(s.len()));
                    let cmds = cmds.into_iter().intersperse(" | ");
                    let cmds: String = if *cmd == CmdDiscriminants::default() {
                        once("[").chain(cmds).chain(once("]")).collect()
                    } else {
                        cmds.collect()
                    };
                    (cmds, arg, descr)
                }).collect()
            };
            let cmd_len = data.iter().map(|d| d.0.len()).max().unwrap();
            let arg_len = data.iter().map(|d| d.1.len()).max().unwrap();
            let mut cmd_list = String::new();
            for (cmd, arg, descr) in data {
                writeln!(cmd_list, "    - {} {}: {}", cmd.pad_to_width(cmd_len), arg.pad_to_width(arg_len), descr).unwrap();
            }
            cmd_list
        };
        format!("{descr}\n\nAvailable commands:\n{cmd_list}\nMade by {authors}").leak()
    };
}
