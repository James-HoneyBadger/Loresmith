use crate::game_state::{AdventureGame, Monster, MonsterStatus, name_matches};
use crate::systems::System;

#[derive(Debug, Default)]
pub struct BasicWorldSystem;

impl BasicWorldSystem {
    /// Expand single-letter direction abbreviations to full words so exit
    /// keys in the adventure JSON ("north", "south" …) are matched reliably.
    fn expand_direction(dir: &str) -> &str {
        match dir {
            "n" => "north",
            "s" => "south",
            "e" => "east",
            "w" => "west",
            "u" => "up",
            "d" => "down",
            other => other,
        }
    }

    fn find_npc<'a>(game: &'a AdventureGame, query: &str) -> Option<&'a Monster> {
        game.get_monsters_in_room(game.player.current_room)
            .into_iter()
            .find(|m| name_matches(&m.name, query))
    }

    fn short_description(text: &str) -> &str {
        text.split(['.', '!', '?'])
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("I don't have much to say right now")
    }

    fn talk_to_npc(game: &mut AdventureGame, target: &str) -> String {
        let Some(npc) = Self::find_npc(game, target) else {
            return format!("There's no {} here to talk to.", target);
        };

        let npc_name = npc.name.clone();
        let npc_description = npc.description.clone();
        let friendliness = npc.friendliness.clone();

        game.turn_count += 1;
        match friendliness {
            MonsterStatus::Friendly => format!(
                "{} says: \"{}\"",
                npc_name,
                Self::short_description(&npc_description)
            ),
            MonsterStatus::Neutral => format!(
                "{} eyes you cautiously and says: \"{}\"",
                npc_name,
                Self::short_description(&npc_description)
            ),
            MonsterStatus::Hostile => {
                format!("{} snarls and refuses to speak peacefully.", npc_name)
            }
        }
    }

    fn ask_npc(game: &mut AdventureGame, target: &str, topic: &str) -> String {
        let Some(npc) = Self::find_npc(game, target) else {
            return format!("There's no {} here to ask.", target);
        };

        let npc_name = npc.name.clone();
        let npc_description = npc.description.clone();
        let friendliness = npc.friendliness.clone();

        game.turn_count += 1;
        match friendliness {
            MonsterStatus::Friendly => format!(
                "{} says: \"About {}? {}.\"",
                npc_name,
                topic,
                Self::short_description(&npc_description)
            ),
            MonsterStatus::Neutral => format!(
                "{} says: \"About {}? {}.\"",
                npc_name,
                topic,
                Self::short_description(&npc_description)
            ),
            MonsterStatus::Hostile => format!(
                "{} growls: \"Ask me about {} again and you'll regret it.\"",
                npc_name, topic
            ),
        }
    }
}

impl System for BasicWorldSystem {
    fn on_command(
        &mut self,
        command: &str,
        args: &[&str],
        game: &mut AdventureGame,
    ) -> Option<String> {
        match command {
            "help" | "?" => Some(Self::help_text()),
            "look" | "l" => Some(game.look()),
            "save" => {
                let path = if args.is_empty() {
                    "savegame.json".to_string()
                } else {
                    args.join(" ")
                };
                match game.save_game(&path) {
                    Ok(()) => Some(format!("Game saved to {}.", path)),
                    Err(err) => Some(format!("Failed to save game: {}", err)),
                }
            }
            "autosave" => match args.first().copied() {
                None | Some("status") => Some(game.autosave_status()),
                Some("off") | Some("disable") => match game.set_autosave(false, None) {
                    Ok(msg) => Some(msg),
                    Err(err) => Some(format!("Failed to change autosave: {}", err)),
                },
                Some("on") | Some("enable") => {
                    let path = args.iter().skip(1).copied().collect::<Vec<_>>().join(" ");
                    let path = if path.trim().is_empty() {
                        None
                    } else {
                        Some(path)
                    };
                    match game.set_autosave(true, path) {
                        Ok(msg) => Some(msg),
                        Err(err) => Some(format!("Failed to enable autosave: {}", err)),
                    }
                }
                Some(_) => {
                    let path = args.join(" ");
                    match game.set_autosave(true, Some(path)) {
                        Ok(msg) => Some(msg),
                        Err(err) => Some(format!("Failed to enable autosave: {}", err)),
                    }
                }
            },
            "load" => {
                let path = if args.is_empty() {
                    "savegame.json".to_string()
                } else {
                    args.join(" ")
                };
                match game.load_game(&path) {
                    Ok(_) => Some(format!("Game loaded from {}.\n{}", path, game.look())),
                    Err(err) => Some(format!("Failed to load game: {}", err)),
                }
            }
            "go" | "move" => {
                if let Some(dir) = args.first() {
                    let full = Self::expand_direction(dir);
                    match game.move_player(full) {
                        Some(desc) => Some(desc),
                        None => Some(format!("You can't go {}.", full)),
                    }
                } else {
                    Some("Go where?".to_string())
                }
            }
            dir if [
                "north", "south", "east", "west", "up", "down", "n", "s", "e", "w", "u", "d",
            ]
            .contains(&dir) =>
            {
                let full = Self::expand_direction(dir);
                match game.move_player(full) {
                    Some(desc) => Some(desc),
                    None => Some("You can't go that way.".to_string()),
                }
            }
            "talk" | "speak" | "greet" | "hello" => {
                if args.is_empty() {
                    Some("Talk to whom?".to_string())
                } else {
                    let target = if args.first() == Some(&"to") {
                        args.iter().skip(1).copied().collect::<Vec<_>>().join(" ")
                    } else {
                        args.join(" ")
                    };
                    if target.trim().is_empty() {
                        Some("Talk to whom?".to_string())
                    } else {
                        Some(Self::talk_to_npc(game, &target))
                    }
                }
            }
            "ask" => {
                if args.is_empty() {
                    Some("Ask whom about what?".to_string())
                } else if let Some(about_index) = args.iter().position(|arg| *arg == "about") {
                    let target = args[..about_index].join(" ");
                    let topic = args[about_index + 1..].join(" ");
                    if target.trim().is_empty() || topic.trim().is_empty() {
                        Some("Usage: ask <npc> about <topic>".to_string())
                    } else {
                        Some(Self::ask_npc(game, &target, &topic))
                    }
                } else {
                    let target = args.join(" ");
                    Some(Self::talk_to_npc(game, &target))
                }
            }
            "say" | "shout" | "yell" => {
                let text = args.join(" ");
                if text.is_empty() {
                    Some("Say what?".to_string())
                } else {
                    // Collect all non-hostile NPCs in the room
                    let monsters = game.get_monsters_in_room(game.player.current_room);
                    let npc_names: Vec<String> = monsters
                        .iter()
                        .filter(|m| m.friendliness != MonsterStatus::Hostile)
                        .map(|m| m.name.clone())
                        .collect();
                    let mut response = format!("You say: \"{}\"", text);
                    if !npc_names.is_empty() {
                        for npc in &npc_names {
                            response.push_str(&format!("\n{} turns to face you.", npc));
                        }
                    }
                    Some(response)
                }
            }
            _ => None,
        }
    }
}

impl BasicWorldSystem {
    fn help_text() -> String {
        [
            "Commands:",
            "  look / l                    Look around",
            "  inventory / i / inv         Show inventory",
            "  n/s/e/w/u/d                 Move in a direction",
            "  take <item>                 Pick up an item",
            "  drop <item>                 Drop an item",
            "  equip/wield/wear <item>     Equip a weapon or armor",
            "  unequip/remove <slot>       Unequip weapon or armor",
            "  use <item>                  Use/consume an item",
            "  examine / x <item>          Examine an item",
            "  attack / fight <monster>    Attack a monster",
            "  flee / run                  Attempt to flee combat",
            "  talk <npc>                  Speak directly to someone nearby",
            "  ask <npc> about <topic>     Ask an NPC about a subject",
            "  say / shout / yell <text>   Speak aloud",
            "  status / stats              Show player status & XP",
            "  quests / journal            Show quest journal",
            "  accept <quest_id>           Accept a quest",
            "  complete <quest_id>         Complete a quest",
            "  save [file]                 Save current progress",
            "  load [file]                 Load saved progress",
            "  autosave on [file]          Enable automatic saving",
            "  autosave off                Disable automatic saving",
            "  autosave status             Show autosave status",
            "  help / ?                    Show this help",
        ]
        .join("\n")
    }
}
