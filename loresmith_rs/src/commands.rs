//! Single source of truth for command help text.
//!
//! Frontends and documentation derive their command lists from this table so
//! aliases cannot drift from the systems that implement them. `help` renders it
//! in-game, and `README.md` / `release/README.md` embed the Markdown form; the
//! integration tests in `tests/commands.rs` fail if either drifts.

/// One command entry: every alias that triggers it, its argument hint, and a description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandHelp {
    pub aliases: &'static [&'static str],
    pub args: &'static str,
    pub description: &'static str,
}

impl CommandHelp {
    /// Render the aliases and argument hint, for example `take/get/grab/pick <item>`.
    pub fn usage(&self) -> String {
        if self.args.is_empty() {
            self.aliases.join("/")
        } else {
            format!("{} {}", self.aliases.join("/"), self.args)
        }
    }
}

/// Commands implemented by the five built-in systems.
pub const COMMANDS: &[CommandHelp] = &[
    CommandHelp {
        aliases: &["look", "l"],
        args: "",
        description: "Look around",
    },
    CommandHelp {
        aliases: &["inventory", "i", "inv"],
        args: "",
        description: "Show inventory",
    },
    CommandHelp {
        aliases: &["n", "s", "e", "w", "u", "d"],
        args: "",
        description: "Move in a compass direction",
    },
    CommandHelp {
        aliases: &["north", "south", "east", "west", "up", "down"],
        args: "",
        description: "Move using a full direction word",
    },
    CommandHelp {
        aliases: &["go", "move"],
        args: "<direction>",
        description: "Move in a direction",
    },
    CommandHelp {
        aliases: &["take", "get", "grab", "pick"],
        args: "<item>",
        description: "Pick up an item",
    },
    CommandHelp {
        aliases: &["drop"],
        args: "<item>",
        description: "Drop an item",
    },
    CommandHelp {
        aliases: &["equip", "wield", "wear"],
        args: "<item>",
        description: "Equip a weapon or armor",
    },
    CommandHelp {
        aliases: &["unequip", "remove"],
        args: "<slot>",
        description: "Unequip weapon or armor",
    },
    CommandHelp {
        aliases: &["use", "consume", "eat", "drink"],
        args: "<item>",
        description: "Use, consume, or read an item",
    },
    CommandHelp {
        aliases: &["examine", "inspect", "x"],
        args: "<item>",
        description: "Examine an item",
    },
    CommandHelp {
        aliases: &["attack", "fight", "kill"],
        args: "<monster>",
        description: "Attack a hostile monster",
    },
    CommandHelp {
        aliases: &["flee", "run", "escape"],
        args: "",
        description: "Attempt to flee combat",
    },
    CommandHelp {
        aliases: &["talk", "speak", "greet", "hello"],
        args: "<npc>",
        description: "Speak directly to someone nearby",
    },
    CommandHelp {
        aliases: &["ask"],
        args: "<npc> about <topic>",
        description: "Ask an NPC about a subject",
    },
    CommandHelp {
        aliases: &["say", "shout", "yell"],
        args: "<text>",
        description: "Speak aloud",
    },
    CommandHelp {
        aliases: &["unlock"],
        args: "<direction>",
        description: "Unlock an exit with a carried key",
    },
    CommandHelp {
        aliases: &["shop", "trade", "store"],
        args: "[name]",
        description: "Browse shops in this room, or one shop by name",
    },
    CommandHelp {
        aliases: &["buy", "purchase"],
        args: "<item>",
        description: "Buy from the cheapest shop in this room",
    },
    CommandHelp {
        aliases: &["sell"],
        args: "<item>",
        description: "Sell to the best-paying shop in this room",
    },
    CommandHelp {
        aliases: &["status", "stats", "score"],
        args: "",
        description: "Show player status, XP, and reputation",
    },
    CommandHelp {
        aliases: &["quests", "journal"],
        args: "",
        description: "Show quest journal",
    },
    CommandHelp {
        aliases: &["accept"],
        args: "<quest_id>",
        description: "Accept a quest",
    },
    CommandHelp {
        aliases: &["complete", "finish"],
        args: "<quest_id>",
        description: "Turn in a completed quest",
    },
    CommandHelp {
        aliases: &["abandon"],
        args: "<quest_id>",
        description: "Abandon an active quest",
    },
    CommandHelp {
        aliases: &["save"],
        args: "[file]",
        description: "Save current progress",
    },
    CommandHelp {
        aliases: &["load"],
        args: "[file]",
        description: "Load saved progress",
    },
    CommandHelp {
        aliases: &["saves"],
        args: "",
        description: "List JSON saves in this directory",
    },
    CommandHelp {
        aliases: &["autosave"],
        args: "on [file]",
        description: "Enable automatic saving",
    },
    CommandHelp {
        aliases: &["autosave"],
        args: "off",
        description: "Disable automatic saving",
    },
    CommandHelp {
        aliases: &["autosave"],
        args: "status",
        description: "Show autosave status",
    },
    CommandHelp {
        aliases: &["restart"],
        args: "",
        description: "Start the adventure over",
    },
    CommandHelp {
        aliases: &["help", "?"],
        args: "",
        description: "Show this help",
    },
];

/// Render the command table for the `help` command.
pub fn help_text() -> String {
    let mut lines = vec!["Commands:".to_string()];
    let width = COMMANDS
        .iter()
        .map(|command| command.usage().chars().count())
        .max()
        .unwrap_or(0);
    for command in COMMANDS {
        lines.push(format!(
            "  {:<width$}  {}",
            command.usage(),
            command.description,
            width = width
        ));
    }
    lines.join("\n")
}

/// Render the command table as Markdown, for embedding in documentation.
pub fn markdown_table() -> String {
    let mut table = String::from("| Command | Description |\n| --- | --- |\n");
    for command in COMMANDS {
        table.push_str(&format!(
            "| `{}` | {} |\n",
            command.usage(),
            command.description
        ));
    }
    table.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_covers_core_commands() {
        let text = help_text();
        for expected in [
            "take/get/grab/pick",
            "use/consume/eat/drink",
            "talk/speak/greet/hello",
            "ask <npc> about <topic>",
            "shop/trade/store",
            "buy/purchase <item>",
            "unlock <direction>",
            "restart",
        ] {
            assert!(text.contains(expected), "help is missing {expected}");
        }
    }

    #[test]
    fn every_entry_has_a_verb_and_description() {
        for command in COMMANDS {
            assert!(
                !command.aliases.is_empty(),
                "an entry has no aliases: {:?}",
                command.description
            );
            assert!(
                !command.description.trim().is_empty(),
                "{} has no description",
                command.usage()
            );
            for alias in command.aliases {
                assert!(
                    !alias.is_empty() && alias.chars().all(|c| c.is_ascii_lowercase() || c == '?'),
                    "alias '{alias}' should be a bare lowercase verb"
                );
            }
        }
    }

    #[test]
    fn markdown_table_lists_every_entry() {
        let table = markdown_table();
        assert_eq!(table.lines().count(), COMMANDS.len() + 2);
        for command in COMMANDS {
            assert!(
                table.contains(&format!("`{}`", command.usage())),
                "markdown table is missing {}",
                command.usage()
            );
        }
    }

    /// Prints the Markdown table for pasting into the READMEs.
    ///
    /// Run with:
    /// `cargo test -p loresmith_rs print_markdown_table -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn print_markdown_table() {
        println!("{}", markdown_table());
    }
}
