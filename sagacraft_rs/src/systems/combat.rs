use crate::game_state::{AdventureGame, GameEvent, MonsterStatus, award_experience, name_matches};
use crate::systems::System;
use rand::Rng;

#[derive(Debug, Default)]
pub struct CombatSystem;

impl System for CombatSystem {
    fn on_command(
        &mut self,
        command: &str,
        args: &[&str],
        game: &mut AdventureGame,
    ) -> Option<String> {
        match command {
            "attack" | "fight" | "kill" => {
                if let Some(target) = args.first() {
                    self.attack_monster(game, target)
                } else {
                    Some("Attack what?".to_string())
                }
            }
            "status" | "stats" | "score" => Some(self.show_status(game)),
            "flee" | "run" | "escape" => Some(self.flee(game)),
            _ => None,
        }
    }
}

impl CombatSystem {
    fn attack_monster(&self, game: &mut AdventureGame, target_name: &str) -> Option<String> {
        // Collect matching monster id first to avoid borrow conflicts
        let monster_id = game
            .get_monsters_in_room(game.player.current_room)
            .iter()
            .find(|m| name_matches(&m.name, target_name))
            .map(|m| m.id);

        let Some(monster_id) = monster_id else {
            return Some(format!("There's no {} here to attack.", target_name));
        };

        // Don't allow attacking non-hostile NPCs
        if let Some(m) = game.monsters.get(&monster_id)
            && m.friendliness != MonsterStatus::Hostile
        {
            let disposition = match m.friendliness {
                MonsterStatus::Friendly => "friendly",
                MonsterStatus::Neutral => "peaceful",
                MonsterStatus::Hostile => "hostile",
            };
            return Some(format!(
                "You can't bring yourself to attack the {} {}.",
                disposition, m.name
            ));
        }

        // Weapon dice plus a small skill bonus from `weapon_ability`.
        let player_damage = if let Some(weapon_id) = game.player.equipped_weapon {
            if let Some(weapon) = game.items.get(&weapon_id) {
                let skill = game
                    .player
                    .weapon_ability
                    .get(&weapon.weapon_type)
                    .copied()
                    .unwrap_or(5);
                weapon.get_damage() + skill / 5
            } else {
                rand::rng().random_range(1..=4)
            }
        } else {
            let best = game
                .player
                .weapon_ability
                .values()
                .copied()
                .max()
                .unwrap_or(4);
            rand::rng().random_range(1..=best.max(4)) + best / 5
        };

        let mut output = String::new();
        let mut kill_info: Option<(String, i32, i32, i32)> = None;

        // Apply player's attack to monster; monster armor reduces damage
        if let Some(monster) = game.monsters.get_mut(&monster_id) {
            let armor_reduction = monster.armor_worn;
            let net_damage = (player_damage - armor_reduction).max(1);
            monster.current_health -= net_damage;

            if armor_reduction > 0 {
                output.push_str(&format!(
                    "You attack the {} for {} damage ({} absorbed by armor).",
                    monster.name, net_damage, armor_reduction
                ));
            } else {
                output.push_str(&format!(
                    "You attack the {} for {} damage.",
                    monster.name, net_damage
                ));
            }

            if monster.current_health <= 0 {
                monster.is_dead = true;
                kill_info = Some((
                    monster.name.clone(),
                    monster.room_id,
                    monster.gold,
                    monster.hardiness * 5,
                ));
            } else {
                output.push_str(&format!(
                    " It has {} health remaining.",
                    monster.current_health
                ));
            }
        } else {
            return Some(format!("There's no {} here to attack.", target_name));
        }

        if let Some((name, room_id, gold, xp_gained)) = kill_info {
            game.player.gold += gold;
            game.turn_count += 1;

            let mut msg = format!("You defeat the {}!", name);
            if gold > 0 {
                msg.push_str(&format!(" (+{} gold)", gold));
            }
            if xp_gained > 0 {
                msg.push_str(&format!(" (+{} XP)", xp_gained));
            }
            for level_message in award_experience(game, xp_gained) {
                msg.push('\n');
                msg.push_str(&level_message);
            }
            if let Some(loot_message) = Self::drop_loot(game, monster_id) {
                msg.push('\n');
                msg.push_str(&loot_message);
            }
            game.events.push(GameEvent::MonsterKilled {
                monster_name: name,
                monster_id,
                room_id,
            });
            if let Some(ending) = game.check_ending() {
                msg.push('\n');
                msg.push_str(&ending);
            }
            return Some(msg);
        }

        // Monster counter-attack (if still alive)
        let counter = self.monster_counter_attack(game, monster_id);
        output.push('\n');
        output.push_str(&counter);
        game.turn_count += 1;

        Some(output)
    }

    fn monster_counter_attack(&self, game: &mut AdventureGame, monster_id: i32) -> String {
        // Determine monster's attack damage: use its weapon if it has one, else agility-based formula
        let (monster_dmg, monster_name) = if let Some(m) = game.monsters.get(&monster_id) {
            let dmg = if let Some(weapon_id) = m.weapon_id {
                // Use the weapon's damage if the item exists, otherwise fall back
                if let Some(weapon) = game.items.get(&weapon_id) {
                    weapon.get_damage()
                } else {
                    let max_dmg = (m.agility / 3 + 1).max(2);
                    rand::rng().random_range(1..=max_dmg)
                }
            } else {
                let max_dmg = (m.agility / 3 + 1).max(2);
                rand::rng().random_range(1..=max_dmg)
            };
            (dmg, m.name.clone())
        } else {
            return String::new();
        };

        // Reduce by equipped armor plus a small armor-expertise bonus.
        let armor_reduction = if let Some(armor_id) = game.player.equipped_armor {
            game.items.get(&armor_id).map_or(0, |a| a.armor_value)
        } else {
            0
        } + game.player.armor_expertise / 5;
        let net_damage = (monster_dmg - armor_reduction).max(1);

        game.player.current_health -= net_damage;
        let current_hp = game.player.current_health;

        if current_hp <= 0 {
            game.game_over = true;
            if let Some(text) = game
                .document
                .ending
                .as_ref()
                .and_then(|ending| ending.defeat_text.clone())
            {
                game.ending_text = text;
            }
            format!(
                "The {} strikes back for {} damage. You have been slain!",
                monster_name, net_damage
            )
        } else {
            format!(
                "The {} strikes back for {} damage. Your health: {}/{}.",
                monster_name, net_damage, current_hp, game.player.hardiness
            )
        }
    }

    fn flee(&self, game: &mut AdventureGame) -> String {
        let has_hostiles = game
            .get_monsters_in_room(game.player.current_room)
            .into_iter()
            .any(|m| m.friendliness == MonsterStatus::Hostile);

        if !has_hostiles {
            return "You aren't in combat — there's nothing to flee from.".to_string();
        }

        // Flee success chance based on player agility (10% – 90%) and the
        // courage of the most determined pursuer.
        let max_courage = game
            .get_monsters_in_room(game.player.current_room)
            .into_iter()
            .filter(|monster| monster.friendliness == MonsterStatus::Hostile)
            .map(|monster| monster.courage)
            .max()
            .unwrap_or(0);
        let flee_chance =
            (game.player.agility as f32 / 20.0 - max_courage as f32 / 1000.0).clamp(0.10, 0.90);

        if rand::random::<f32>() < flee_chance {
            let mut exits: Vec<(String, i32)> = game
                .get_current_room()
                .map(|room| {
                    room.exits
                        .iter()
                        .map(|(direction, destination)| (direction.clone(), *destination))
                        .collect()
                })
                .unwrap_or_default();
            exits.sort_by(|left, right| left.0.cmp(&right.0));

            let exit = exits
                .iter()
                .find(|(_, destination)| {
                    !Self::room_has_hostiles(game, *destination) && *destination != game.last_room
                })
                .or_else(|| {
                    exits
                        .iter()
                        .find(|(_, destination)| !Self::room_has_hostiles(game, *destination))
                })
                .or_else(|| {
                    exits
                        .iter()
                        .find(|(_, destination)| *destination != game.last_room)
                })
                .or_else(|| exits.first())
                .cloned();

            if let Some((direction, destination)) = exit
                && game.rooms.contains_key(&destination)
            {
                game.last_room = game.player.current_room;
                game.player.current_room = destination;
                game.turn_count += 1;
                game.visited_rooms.insert(destination);
                game.events.push(GameEvent::RoomEntered {
                    room_id: destination,
                });
                let mut message = format!("You flee {}!\n{}", direction, game.look());
                if let Some(ending) = game.check_ending() {
                    message.push('\n');
                    message.push_str(&ending);
                }
                return message;
            }
            "You try to flee but have nowhere to go!".to_string()
        } else {
            // Failed flee: first hostile monster gets a free attack
            let monster_id = game
                .get_monsters_in_room(game.player.current_room)
                .into_iter()
                .find(|m| m.friendliness == MonsterStatus::Hostile)
                .map(|m| m.id);
            if let Some(mid) = monster_id {
                let counter = self.monster_counter_attack(game, mid);
                game.turn_count += 1;
                format!("You fail to flee!\n{}", counter)
            } else {
                "You fail to flee!".to_string()
            }
        }
    }

    fn room_has_hostiles(game: &AdventureGame, room_id: i32) -> bool {
        game.monsters.values().any(|monster| {
            monster.room_id == room_id
                && !monster.is_dead
                && monster.friendliness == MonsterStatus::Hostile
        })
    }

    /// Move a defeated monster's loot and weapon into its room.
    fn drop_loot(game: &mut AdventureGame, monster_id: i32) -> Option<String> {
        let (room_id, loot, weapon_id) = {
            let monster = game.monsters.get(&monster_id)?;
            (monster.room_id, monster.loot.clone(), monster.weapon_id)
        };

        let mut seen = std::collections::HashSet::new();
        let mut dropped = Vec::new();
        for item_id in loot.into_iter().chain(weapon_id) {
            if !seen.insert(item_id) {
                continue;
            }
            if let Some(item) = game.items.get_mut(&item_id) {
                item.location = room_id;
                item.is_takeable = true;
                dropped.push(item.name.clone());
            }
        }

        if dropped.is_empty() {
            None
        } else {
            Some(format!("It drops: {}.", dropped.join(", ")))
        }
    }

    fn show_status(&self, game: &AdventureGame) -> String {
        let weapon_name = game
            .player
            .equipped_weapon
            .and_then(|id| game.items.get(&id))
            .map(|w| w.name.as_str())
            .unwrap_or("none");
        let armor_name = game
            .player
            .equipped_armor
            .and_then(|id| game.items.get(&id))
            .map(|a| a.name.as_str())
            .unwrap_or("none");
        let (carry_cur, carry_max) = game.carry_weight();
        let next_level_xp = game.player.level.saturating_mul(100);
        let location = game
            .get_current_room()
            .map(|room| format!("{} (room {})", room.name, room.id))
            .unwrap_or_else(|| format!("Room {}", game.player.current_room));
        let mut status = format!(
            "Player: {}\nHealth: {}/{}\nLevel: {}  XP: {}/{}\nGold: {}\nWeapon: {}\nArmor: {}\nCarrying: {}/{} weight\nLocation: {}",
            game.player.name,
            game.player.current_health,
            game.player.hardiness,
            game.player.level,
            game.player.experience_points,
            next_level_xp,
            game.player.gold,
            weapon_name,
            armor_name,
            carry_cur,
            carry_max,
            location,
        );
        if !game.player.reputation.is_empty() {
            let mut factions: Vec<_> = game.player.reputation.iter().collect();
            factions.sort_by(|left, right| left.0.cmp(right.0));
            let text = factions
                .into_iter()
                .map(|(faction, value)| format!("{} {:+}", faction, value))
                .collect::<Vec<_>>()
                .join(", ");
            status.push_str(&format!("\nReputation: {}", text));
        }
        status
    }
}
