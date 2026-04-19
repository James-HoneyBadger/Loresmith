use crate::game_state::{AdventureGame, GameEvent, MonsterStatus, name_matches};
use crate::systems::System;
use rand::Rng;

#[derive(Debug, Default)]
pub struct CombatSystem;

/// XP needed to level up: level * 100 (level 1→2 needs 100 XP, level 2→3 needs 200, etc.)
const XP_PER_LEVEL: i32 = 100;

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
            return Some(format!(
                "You can't bring yourself to attack the friendly {}.",
                m.name
            ));
        }

        // Determine player damage using equipped weapon, or unarmed fallback
        let player_damage = if let Some(weapon_id) = game.player.equipped_weapon {
            if let Some(weapon) = game.items.get(&weapon_id) {
                weapon.get_damage()
            } else {
                rand::thread_rng().gen_range(1..=4)
            }
        } else {
            let best = game
                .player
                .weapon_ability
                .values()
                .copied()
                .max()
                .unwrap_or(4);
            rand::thread_rng().gen_range(1..=best.max(4))
        };

        let mut output = String::new();

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
                let name = monster.name.clone();
                let room_id = monster.room_id;
                let gold = monster.gold;
                let xp_gained = monster.hardiness * 5;
                game.player.gold += gold;
                game.player.experience_points += xp_gained;
                game.turn_count += 1;

                let mut msg = format!("You defeat the {}!", name);
                if gold > 0 {
                    msg.push_str(&format!(" (+{} gold)", gold));
                }
                msg.push_str(&format!(" (+{} XP)", xp_gained));
                // Check for level-up
                let level_up_msg = Self::check_level_up(game);
                if let Some(lu) = level_up_msg {
                    msg.push('\n');
                    msg.push_str(&lu);
                }
                game.events.push(GameEvent::MonsterKilled {
                    monster_name: name,
                    room_id,
                });
                return Some(msg);
            } else {
                output.push_str(&format!(
                    " It has {} health remaining.",
                    monster.current_health
                ));
            }
        } else {
            return Some(format!("There's no {} here to attack.", target_name));
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
                    rand::thread_rng().gen_range(1..=max_dmg)
                }
            } else {
                let max_dmg = (m.agility / 3 + 1).max(2);
                rand::thread_rng().gen_range(1..=max_dmg)
            };
            (dmg, m.name.clone())
        } else {
            return String::new();
        };

        // Reduce by player armor
        let armor_reduction = if let Some(armor_id) = game.player.equipped_armor {
            game.items.get(&armor_id).map_or(0, |a| a.armor_value)
        } else {
            0
        };
        let net_damage = (monster_dmg - armor_reduction).max(1);

        game.player.current_health -= net_damage;
        let current_hp = game.player.current_health;

        if current_hp <= 0 {
            game.game_over = true;
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

        // Flee success chance based on player agility (10% – 90%)
        let flee_chance = (game.player.agility as f32 / 20.0).clamp(0.10, 0.90);
        if rand::random::<f32>() < flee_chance {
            // Choose the first available exit
            let exit = game.get_current_room().and_then(|r| {
                r.exits
                    .iter()
                    .next()
                    .map(|(dir, &dest)| (dir.clone(), dest))
            });
            if let Some((dir, dest_id)) = exit
                && game.rooms.contains_key(&dest_id)
            {
                game.player.current_room = dest_id;
                game.turn_count += 1;
                game.events
                    .push(GameEvent::RoomEntered { room_id: dest_id });
                return format!("You flee {}!\n{}", dir, game.look());
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

    /// Check whether the player should level up and apply it.
    fn check_level_up(game: &mut AdventureGame) -> Option<String> {
        let threshold = game.player.level * XP_PER_LEVEL;
        if game.player.experience_points >= threshold {
            game.player.level += 1;
            // Increase stats on level-up
            game.player.hardiness += 2;
            game.player.agility += 1;
            // Restore health to new max
            game.player.current_health = game.player.hardiness;
            Some(format!(
                "*** Level Up! You are now level {}. Hardiness +2, Agility +1. Health restored to {}. ***",
                game.player.level, game.player.hardiness
            ))
        } else {
            None
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
        let next_level_xp = game.player.level * 100;
        format!(
            "Player: {}\nHealth: {}/{}\nLevel: {}  XP: {}/{}\nGold: {}\nWeapon: {}\nArmor: {}\nCarrying: {}/{} weight\nLocation: Room {}",
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
            game.player.current_room,
        )
    }
}
