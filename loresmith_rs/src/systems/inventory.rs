use crate::game_state::AdventureGame;
use crate::systems::System;

#[derive(Debug, Default)]
pub struct InventorySystem;

impl System for InventorySystem {
    fn on_command(
        &mut self,
        command: &str,
        args: &[&str],
        game: &mut AdventureGame,
    ) -> Option<String> {
        match command {
            "inventory" | "inv" | "i" => {
                if game.player.inventory.is_empty() {
                    Some("Your inventory is empty.".to_string())
                } else {
                    let (cur, max) = game.carry_weight();
                    let mut result = format!("Inventory ({}/{} weight):\n", cur, max);
                    for &item_id in &game.player.inventory {
                        if let Some(item) = game.items.get(&item_id) {
                            let equipped = if game.player.equipped_weapon == Some(item_id) {
                                " [wielded]"
                            } else if game.player.equipped_armor == Some(item_id) {
                                " [worn]"
                            } else {
                                ""
                            };
                            result.push_str(&format!("  - {}{}\n", item.name, equipped));
                        }
                    }
                    Some(result.trim_end().to_string())
                }
            }
            "take" | "get" | "grab" | "pick" => {
                let item_name = args.join(" ");
                if item_name.is_empty() {
                    Some("Take what?".to_string())
                } else {
                    Some(game.take_item(&item_name).unwrap_or_else(|e| e))
                }
            }
            "drop" => {
                let item_name = args.join(" ");
                if item_name.is_empty() {
                    Some("Drop what?".to_string())
                } else {
                    match game.drop_item(&item_name) {
                        Some(name) => Some(format!("Dropped: {}.", name)),
                        None => Some("You don't have that.".to_string()),
                    }
                }
            }
            "equip" | "wield" | "wear" => {
                let item_name = args.join(" ");
                if item_name.is_empty() {
                    Some("Equip what?".to_string())
                } else {
                    Some(game.equip_item(&item_name).unwrap_or_else(|e| e))
                }
            }
            "unequip" | "remove" => match args.first().copied() {
                None => Some("Unequip what? Specify 'weapon' or 'armor'.".to_string()),
                Some(slot) => Some(
                    game.unequip_slot(&slot.to_lowercase())
                        .unwrap_or_else(|e| e),
                ),
            },
            "use" | "consume" | "eat" | "drink" => {
                let item_name = args.join(" ");
                if item_name.is_empty() {
                    Some("Use what?".to_string())
                } else {
                    Some(game.use_item(&item_name).unwrap_or_else(|e| e))
                }
            }
            "examine" | "inspect" | "x" => {
                let item_name = args.join(" ");
                if item_name.is_empty() {
                    Some("Examine what?".to_string())
                } else {
                    Some(
                        game.examine_item(&item_name)
                            .unwrap_or_else(|| format!("You don't see any '{}' here.", item_name)),
                    )
                }
            }
            _ => None,
        }
    }
}
