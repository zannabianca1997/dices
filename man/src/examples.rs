//! Parsing of manual examples

use std::iter::once;

use lazy_regex::regex_captures_iter;

/// An example in the manual
pub struct Example<'a> {
    /// Tags of the example
    pub tags: Vec<&'a str>,
    /// Commands in the example
    pub commands: Vec<Command<'a>>,
}

/// A single command in an example
pub struct Command<'a> {
    /// Command starts with `#>>` and should not be printed.
    pub hidden: bool,
    /// The command lines
    ///
    /// ```dices
    /// >>> {line 1}
    /// ... {line 2}
    /// ```
    pub command: Vec<&'a str>,
    /// Response lines
    pub response: &'a str,
}

impl<'a> Command<'a> {
    pub fn command(&self) -> String {
        self.command.join("\n")
    }
}

impl<'a> Example<'a> {
    pub fn new(tags: &'a str, content: &'a str) -> Self {
        let tags = tags
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        let commands = regex_captures_iter!(
            // Parse a command example and the following response
            r"([#>])>>((?:.|\n\.\.\.)*)((?:\n(?:[^#>]|[#>][^>]|[#>]>[^>]).*)*)",
            content
        )
        .map(|c| {
            let (_, [hidden_char, command, response]) = c.extract();
            let mut command: Vec<&str> = command.split("\n...").collect();

            // cut all common whitespace from the lines
            let (first, others) = command.split_first_mut().unwrap();
            while let Some(ch) = first.chars().next()
                && ch.is_whitespace()
                && others.iter().all(|s| s.starts_with(ch))
            {
                for line in once(&mut *first).chain(&mut *others) {
                    *line = &line[ch.len_utf8()..]
                }
            }

            Command {
                hidden: match hidden_char {
                    "#" => true,
                    ">" => false,
                    _ => unreachable!(),
                },
                command,
                response,
            }
        })
        .collect();

        Self { tags, commands }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_content() {
        let example = Example::new("tag", "");
        assert_eq!(example.tags, vec!["tag"]);
        assert!(example.commands.is_empty());
    }

    #[test]
    fn single_visible_command() {
        let example = Example::new("tag", ">>> cmd\n");
        assert_eq!(example.commands.len(), 1);
        let cmd = &example.commands[0];
        assert!(!cmd.hidden);
        assert_eq!(cmd.command, vec!["cmd"]);
        assert_eq!(cmd.response, "");
    }

    #[test]
    fn single_hidden_command() {
        let example = Example::new("tag", "#>> secret\n");
        assert_eq!(example.commands.len(), 1);
        let cmd = &example.commands[0];
        assert!(cmd.hidden);
        assert_eq!(cmd.command, vec!["secret"]);
        assert_eq!(cmd.response, "");
    }

    #[test]
    fn multi_line_command() {
        let example = Example::new("tag", ">>> line1\n... line2\n");
        assert_eq!(example.commands.len(), 1);
        let cmd = &example.commands[0];
        assert!(!cmd.hidden);
        assert_eq!(cmd.command, vec!["line1", "line2"]);
        assert_eq!(cmd.response, "");
    }

    #[test]
    fn command_with_response() {
        let example = Example::new("tag", ">>> cmd\nresponse\nmore response\n");
        assert_eq!(example.commands.len(), 1);
        let cmd = &example.commands[0];
        assert!(!cmd.hidden);
        assert_eq!(cmd.command, vec!["cmd"]);
        assert_eq!(cmd.response, "\nresponse\nmore response");
    }

    #[test]
    fn whitespace_trimming() {
        let example = Example::new("tag", ">>>   line1\n...   line2\n");
        assert_eq!(example.commands.len(), 1);
        let cmd = &example.commands[0];
        assert_eq!(cmd.command, vec!["line1", "line2"]);
    }

    #[test]
    fn mixed_indent_no_trim_beyond_common() {
        let example = Example::new("tag", ">>>   line1\n... line2\n");
        let cmd = &example.commands[0];
        assert_eq!(cmd.command, vec!["  line1", "line2"]);
    }

    #[test]
    fn multiple_commands() {
        let example = Example::new("tag", ">>> first\nresponse1\n>>> second\nresponse2\n");
        assert_eq!(example.commands.len(), 2);
        assert!(!example.commands[0].hidden);
        assert_eq!(example.commands[0].command, vec!["first"]);
        assert_eq!(example.commands[0].response, "\nresponse1");
        assert!(!example.commands[1].hidden);
        assert_eq!(example.commands[1].command, vec!["second"]);
        assert_eq!(example.commands[1].response, "\nresponse2");
    }

    #[test]
    fn tag_parsing() {
        let example = Example::new("foo, bar,  baz ", ">>> cmd\n");
        assert_eq!(example.tags, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn command_method_joins_lines() {
        let example = Example::new("tag", ">>> line1\n... line2\n");
        let cmd = &example.commands[0];
        assert_eq!(cmd.command(), "line1\nline2");
    }

    #[test]
    fn mixed_hidden_visible() {
        let example = Example::new("tag", ">>> visible\n#>> hidden\n");
        assert_eq!(example.commands.len(), 2);
        assert!(!example.commands[0].hidden);
        assert_eq!(example.commands[0].command, vec!["visible"]);
        assert!(example.commands[1].hidden);
        assert_eq!(example.commands[1].command, vec!["hidden"]);
    }
}
