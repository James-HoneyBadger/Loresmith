use crate::game_state::{AdventureGame, GameEvent, Monster, MonsterStatus, name_matches};
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
        let npc_id = npc.id;
        let npc_description = npc.description.clone();
        let friendliness = npc.friendliness.clone();
        let greeting = npc
            .dialogue
            .as_ref()
            .map(|dialogue| dialogue.greeting.clone())
            .filter(|greeting| !greeting.trim().is_empty());

        game.turn_count += 1;
        game.events.push(GameEvent::TalkedTo {
            npc_name: npc_name.clone(),
            npc_id,
            topic: None,
        });

        match friendliness {
            MonsterStatus::Friendly | MonsterStatus::Neutral => {
                let line = greeting
                    .unwrap_or_else(|| Self::short_description(&npc_description).to_string());
                format!("{} says: \"{}\"", npc_name, line)
            }
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
        let npc_id = npc.id;
        let npc_description = npc.description.clone();
        let friendliness = npc.friendliness.clone();
        let dialogue = npc.dialogue.clone();

        game.turn_count += 1;
        game.events.push(GameEvent::TalkedTo {
            npc_name: npc_name.clone(),
            npc_id,
            topic: Some(topic.to_string()),
        });

        match friendliness {
            MonsterStatus::Friendly | MonsterStatus::Neutral => {
                let query = topic.to_lowercase();
                let answer = dialogue.as_ref().and_then(|dialogue| {
                    dialogue
                        .topics
                        .iter()
                        .find(|(key, _)| {
                            let key = key.to_lowercase();
                            key.contains(&query) || query.contains(&key)
                        })
                        .map(|(_, value)| value.clone())
                });
                let fallback = dialogue
                    .as_ref()
                    .and_then(|dialogue| dialogue.unknown.clone())
                    .unwrap_or_else(|| Self::short_description(&npc_description).to_string());
                format!(
                    "{} says: \"About {}? {}\"",
                    npc_name,
                    topic,
                    answer.unwrap_or(fallback)
                )
            }
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
            "restart" => match game.restart() {
                Ok(_) => Some(format!("The adventure begins anew.\n{}", game.look())),
                Err(err) => Some(format!("Failed to restart: {}", err)),
            },
            "saves" => Some(Self::list_saves()),
            "unlock" => {
                if let Some(direction) = args.first() {
                    match game.unlock_exit(&direction.to_lowercase()) {
                        Ok(message) => Some(message),
                        Err(message) => Some(message),
                    }
                } else {
                    Some("Unlock which direction?".to_string())
                }
            }
            "go" | "move" => {
                if let Some(dir) = args.first() {
                    let lowered = dir.to_lowercase();
                    let full = Self::expand_direction(&lowered);
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
    fn list_saves() -> String {
        let mut files: Vec<(String, u64)> = std::fs::read_dir(".")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    let size = entry.metadata().ok().map(|meta| meta.len()).unwrap_or(0);
                    Some((path.display().to_string(), size))
                } else {
                    None
                }
            })
            .collect();
        files.sort_by(|left, right| left.0.cmp(&right.0));
        if files.is_empty() {
            return "No JSON files found in the current directory.".to_string();
        }
        let mut out = String::from("JSON files in this directory:");
        for (path, size) in files {
            out.push_str(&format!("\n  {} ({} bytes)", path, size));
        }
        out
    }

    fn help_text() -> String {
        crate::commands::help_text()
    }
}
