//! Single source of truth for command help text.
//!
//! Frontends and documentation should derive their command lists from this
//! table so aliases do not drift from the systems that implement them.

/// One command entry in the in-game help.
#[derive(Debug, Clone, Copy)]
pub struct CommandHelp {
    pub usage: &'static str,
    pub description: &'static str,
}

/// Commands implemented by the five built-in systems.
pub const COMMANDS: &[CommandHelp] = &[
    CommandHelp {
        usage: "look / l",
        description: "Look around",
    },
    CommandHelp {
        usage: "inventory / i / inv",
        description: "Show inventory",
    },
    CommandHelp {
        usage: "n/s/e/w/u/d or go/move <direction>",
        description: "Move in a direction",
    },
    CommandHelp {
        usage: "take/get/grab/pick <item>",
        description: "Pick up an item",
    },
    CommandHelp {
        usage: "drop <item>",
        description: "Drop an item",
    },
    CommandHelp {
        usage: "equip/wield/wear <item>",
        description: "Equip a weapon or armor",
    },
    CommandHelp {
        usage: "unequip/remove <slot>",
        description: "Unequip weapon or armor",
    },
    CommandHelp {
        usage: "use/consume/eat/drink <item>",
        description: "Use, consume, or read an item",
    },
    CommandHelp {
        usage: "examine / x <item>",
        description: "Examine an item",
    },
    CommandHelp {
        usage: "attack / fight <monster>",
        description: "Attack a hostile monster",
    },
    CommandHelp {
        usage: "flee / run / escape",
        description: "Attempt to flee combat",
    },
    CommandHelp {
        usage: "talk <npc>",
        description: "Speak directly to someone nearby",
    },
    CommandHelp {
        usage: "ask <npc> about <topic>",
        description: "Ask an NPC about a subject",
    },
    CommandHelp {
        usage: "say / shout / yell <text>",
        description: "Speak aloud",
    },
    CommandHelp {
        usage: "unlock <direction>",
        description: "Unlock an exit with a carried key",
    },
    CommandHelp {
        usage: "shop / trade [name]",
        description: "Browse shops in this room, or one shop by name",
    },
    CommandHelp {
        usage: "buy <item>",
        description: "Buy from the cheapest shop in this room",
    },
    CommandHelp {
        usage: "sell <item>",
        description: "Sell to the best-paying shop in this room",
    },
    CommandHelp {
        usage: "status / stats / score",
        description: "Show player status, XP, and reputation",
    },
    CommandHelp {
        usage: "quests / journal",
        description: "Show quest journal",
    },
    CommandHelp {
        usage: "accept <quest_id>",
        description: "Accept a quest",
    },
    CommandHelp {
        usage: "complete <quest_id>",
        description: "Turn in a completed quest",
    },
    CommandHelp {
        usage: "abandon <quest_id>",
        description: "Abandon an active quest",
    },
    CommandHelp {
        usage: "save [file]",
        description: "Save current progress",
    },
    CommandHelp {
        usage: "load [file]",
        description: "Load saved progress",
    },
    CommandHelp {
        usage: "saves",
        description: "List JSON files in this directory",
    },
    CommandHelp {
        usage: "autosave on [file]",
        description: "Enable automatic saving",
    },
    CommandHelp {
        usage: "autosave off",
        description: "Disable automatic saving",
    },
    CommandHelp {
        usage: "autosave status",
        description: "Show autosave status",
    },
    CommandHelp {
        usage: "restart",
        description: "Start the adventure over",
    },
    CommandHelp {
        usage: "help / ?",
        description: "Show this help",
    },
];

/// Render the command table for the `help` command.
pub fn help_text() -> String {
    let mut lines = vec!["Commands:".to_string()];
    let width = COMMANDS
        .iter()
        .map(|command| command.usage.len())
        .max()
        .unwrap_or(0);
    for command in COMMANDS {
        lines.push(format!(
            "  {:<width$}  {}",
            command.usage,
            command.description,
            width = width
        ));
    }
    lines.join("\n")
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
            "talk <npc>",
            "ask <npc> about <topic>",
            "shop / trade",
            "buy <item>",
            "unlock <direction>",
            "restart",
        ] {
            assert!(text.contains(expected), "help is missing {expected}");
        }
    }
}
