use crate::game_state::{AdventureGame, GameEvent, Shop, ShopStock, name_matches};
use crate::systems::System;

/// Charisma beyond this point no longer changes shop prices. It keeps very high
/// stats from turning buying and selling into an infinite money loop.
const CHARISMA_CAP: i32 = 50;
/// Ignore absurd multipliers in hand-edited adventures.
const MULTIPLIER_CAP: f32 = 10.0;

const SHOP_VERBS: &[&str] = &["shop", "trade", "store"];
const BUY_VERBS: &[&str] = &["buy", "purchase"];
const SELL_VERBS: &[&str] = &["sell"];

#[derive(Debug, Default)]
pub struct ShopSystem;

impl ShopSystem {
    /// Verbs this system answers to, used by the command-reference tests.
    pub const VERB_GROUPS: &'static [&'static [&'static str]] =
        &[SHOP_VERBS, BUY_VERBS, SELL_VERBS];
}

impl ShopSystem {
    fn shops_here(game: &AdventureGame) -> Vec<&Shop> {
        let mut shops: Vec<&Shop> = game
            .shops
            .values()
            .filter(|shop| shop.room_id == game.player.current_room)
            .collect();
        shops.sort_by_key(|shop| shop.id);
        shops
    }

    fn buy_multiplier(shop: &Shop) -> f32 {
        if shop.buy_multiplier.is_finite() {
            shop.buy_multiplier.clamp(0.01, MULTIPLIER_CAP)
        } else {
            0.01
        }
    }

    fn sell_multiplier(shop: &Shop) -> f32 {
        if shop.sell_multiplier.is_finite() {
            shop.sell_multiplier.clamp(0.0, MULTIPLIER_CAP)
        } else {
            0.0
        }
    }

    fn buy_price(shop: &Shop, value: i32, charisma: i32) -> i32 {
        let discount = (charisma.clamp(0, CHARISMA_CAP) as f32 * 0.01).clamp(0.0, 0.5);
        ((value.max(0) as f32) * Self::buy_multiplier(shop) * (1.0 - discount))
            .ceil()
            .max(1.0) as i32
    }

    fn sell_price(shop: &Shop, item_id: i32, value: i32, charisma: i32) -> i32 {
        let bonus = (0.5 + charisma.clamp(0, CHARISMA_CAP) as f32 * 0.01).clamp(0.1, 1.0);
        let offer = ((value.max(0) as f32) * Self::sell_multiplier(shop) * bonus)
            .floor()
            .max(0.0) as i32;
        if shop.stock.iter().any(|stock| stock.item_id == item_id) {
            // Never pay more than this shop would charge for the same item;
            // otherwise a high-charisma character could buy and sell in a loop.
            offer.min(Self::buy_price(shop, value, charisma).saturating_sub(1))
        } else {
            offer
        }
    }

    /// Split a command like `buy Iron Sword 2` into (query, quantity).
    fn parse_query_and_quantity(args: &[&str]) -> (String, i32) {
        if let Some((last, rest)) = args.split_last()
            && let Ok(quantity) = last.parse::<i32>()
            && !rest.is_empty()
        {
            return (rest.join(" "), quantity.max(1));
        }
        (args.join(" "), 1)
    }

    fn list_shop(game: &AdventureGame, args: &[&str]) -> String {
        let shops = Self::shops_here(game);
        if shops.is_empty() {
            return "There is no shop here.".to_string();
        }
        let query = args.join(" ");
        let selected: Vec<&Shop> = if query.trim().is_empty() {
            shops
        } else {
            shops
                .into_iter()
                .filter(|shop| name_matches(&shop.name, &query))
                .collect()
        };
        if selected.is_empty() {
            return format!("There is no shop called '{}' here.", query.trim());
        }
        selected
            .into_iter()
            .map(|shop| Self::describe_shop(game, shop))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn describe_shop(game: &AdventureGame, shop: &Shop) -> String {
        let mut out = format!(
            "{}\n  Shop gold: {} · Your gold: {}\n  Buy rate: x{:.2} value · Sell rate: x{:.2} value · Your charisma: {}",
            shop.name,
            shop.gold,
            game.player.gold,
            Self::buy_multiplier(shop),
            Self::sell_multiplier(shop),
            game.player.charisma
        );

        out.push_str("\n\nFor sale:");
        let mut offered = false;
        for stock in &shop.stock {
            if stock.quantity <= 0 {
                continue;
            }
            let Some(item) = game.items.get(&stock.item_id) else {
                continue;
            };
            let price = Self::buy_price(shop, item.value, game.player.charisma);
            out.push_str(&format!(
                "\n  - {} — {} gold ({} in stock)",
                item.name, price, stock.quantity
            ));
            offered = true;
        }
        if !offered {
            out.push_str("\n  (sold out)");
        }

        out.push_str("\n\nYou can sell:");
        let mut offers: Vec<(String, i32)> = game
            .player
            .inventory
            .iter()
            .filter_map(|id| {
                game.items.get(id).map(|item| {
                    (
                        item.name.clone(),
                        Self::sell_price(shop, item.id, item.value, game.player.charisma),
                    )
                })
            })
            .collect();
        offers.sort_by(|left, right| left.0.cmp(&right.0));
        if offers.is_empty() {
            out.push_str("\n  (nothing)");
        } else {
            for (name, price) in offers {
                let note = if price > shop.gold {
                    " (shop can't afford)"
                } else {
                    ""
                };
                out.push_str(&format!("\n  - {} — {} gold{}", name, price, note));
            }
        }
        out
    }

    fn buy(game: &mut AdventureGame, args: &[&str]) -> String {
        let shops = Self::shops_here(game);
        if shops.is_empty() {
            return "There is no shop here.".to_string();
        }
        let (query, quantity) = Self::parse_query_and_quantity(args);
        if query.trim().is_empty() {
            return "Buy what?".to_string();
        }
        if quantity != 1 {
            return "Items are unique in this engine; buy one at a time.".to_string();
        }

        let mut best: Option<(i32, usize, i32, i32)> = None;
        let mut sold_out: Option<String> = None;
        for shop in &shops {
            for (stock_index, stock) in shop.stock.iter().enumerate() {
                let Some(item) = game.items.get(&stock.item_id) else {
                    continue;
                };
                if !name_matches(&item.name, &query) {
                    continue;
                }
                if stock.quantity <= 0 {
                    sold_out.get_or_insert_with(|| item.name.clone());
                    continue;
                }
                let price = Self::buy_price(shop, item.value, game.player.charisma);
                let better = match best {
                    Some((_, _, _, best_price)) => price < best_price,
                    None => true,
                };
                if better {
                    best = Some((shop.id, stock_index, stock.item_id, price));
                }
            }
        }
        let Some((shop_id, stock_index, item_id, price)) = best else {
            if let Some(name) = sold_out {
                return format!("The shop is sold out of the {}.", name);
            }
            return format!("No shop here stocks '{}'.", query);
        };
        let Some(item) = game.items.get(&item_id) else {
            return "That item is missing from the shop's stock.".to_string();
        };
        let item_name = item.name.clone();
        let item_weight = item.weight;
        let shop_name = game
            .shops
            .get(&shop_id)
            .map(|shop| shop.name.clone())
            .unwrap_or_default();

        if game.player.inventory.contains(&item_id) {
            return format!("You are already carrying the {}.", item_name);
        }
        if game.player.gold < price {
            return format!(
                "You can't afford the {} ({} gold, you have {}).",
                item_name, price, game.player.gold
            );
        }
        let (current_weight, max_weight) = game.carry_weight();
        if current_weight + item_weight > max_weight {
            return format!(
                "The {} is too heavy to carry ({}/{} weight used).",
                item_name, current_weight, max_weight
            );
        }

        game.player.gold -= price;
        game.player.inventory.push(item_id);
        if let Some(item) = game.items.get_mut(&item_id) {
            item.location = 0;
        }
        if let Some(shop) = game.shops.get_mut(&shop_id) {
            shop.gold = shop.gold.saturating_add(price);
            if let Some(stock) = shop.stock.get_mut(stock_index) {
                stock.quantity = stock.quantity.saturating_sub(1);
            }
        }
        game.events.push(GameEvent::ItemPurchased {
            item_name: item_name.clone(),
            item_id,
        });
        // Purchases count as collecting the item for quest objectives.
        game.events.push(GameEvent::ItemCollected {
            item_name: item_name.clone(),
            item_id,
        });
        game.turn_count += 1;
        let mut message = format!(
            "Bought the {} for {} gold at {} ({} gold remaining).",
            item_name, price, shop_name, game.player.gold
        );
        if let Some(ending) = game.check_ending() {
            message.push('\n');
            message.push_str(&ending);
        }
        message
    }

    fn sell(game: &mut AdventureGame, args: &[&str]) -> String {
        let shops = Self::shops_here(game);
        if shops.is_empty() {
            return "There is no shop here.".to_string();
        }
        let (query, quantity) = Self::parse_query_and_quantity(args);
        if query.trim().is_empty() {
            return "Sell what?".to_string();
        }
        if quantity != 1 {
            return "Items are unique in this engine; sell one at a time.".to_string();
        }

        let Some(item_id) = game.player.inventory.iter().copied().find(|id| {
            game.items
                .get(id)
                .is_some_and(|item| name_matches(&item.name, &query))
        }) else {
            return format!("You don't have '{}'.", query);
        };
        let Some(item) = game.items.get(&item_id) else {
            return "That item no longer exists.".to_string();
        };
        let item_name = item.name.clone();
        let item_value = item.value;

        let mut best: Option<(i32, String, i32)> = None;
        let mut unaffordable: Option<(String, i32, i32)> = None;
        for shop in &shops {
            let offer = Self::sell_price(shop, item_id, item_value, game.player.charisma);
            if shop.gold >= offer {
                let better = match &best {
                    Some((_, _, best_price)) => offer > *best_price,
                    None => true,
                };
                if better {
                    best = Some((shop.id, shop.name.clone(), offer));
                }
            } else {
                let better = match &unaffordable {
                    Some((_, best_price, _)) => offer > *best_price,
                    None => true,
                };
                if better {
                    unaffordable = Some((shop.name.clone(), offer, shop.gold));
                }
            }
        }
        let Some((shop_id, shop_name, price)) = best else {
            if let Some((shop_name, offer, gold)) = unaffordable {
                return format!(
                    "No shop here can afford the {} (best offer {} gold; {} has {}).",
                    item_name, offer, shop_name, gold
                );
            }
            return format!("No shop here will buy the {}.", item_name);
        };

        game.player.inventory.retain(|id| *id != item_id);
        if game.player.equipped_weapon == Some(item_id) {
            game.player.equipped_weapon = None;
        }
        if game.player.equipped_armor == Some(item_id) {
            game.player.equipped_armor = None;
        }
        if let Some(item) = game.items.get_mut(&item_id) {
            item.location = -2; // stored in shop inventory, hidden from rooms
        }
        game.player.gold = game.player.gold.saturating_add(price);
        if let Some(shop) = game.shops.get_mut(&shop_id) {
            shop.gold -= price;
            if let Some(stock) = shop.stock.iter_mut().find(|stock| stock.item_id == item_id) {
                stock.quantity = stock.quantity.saturating_add(1);
            } else {
                shop.stock.push(ShopStock {
                    item_id,
                    quantity: 1,
                });
            }
        }
        game.events.push(GameEvent::ItemSold {
            item_name: item_name.clone(),
            item_id,
        });
        game.turn_count += 1;
        format!(
            "Sold the {} to {} for {} gold ({} gold total).",
            item_name, shop_name, price, game.player.gold
        )
    }
}

impl System for ShopSystem {
    fn on_command(
        &mut self,
        command: &str,
        args: &[&str],
        game: &mut AdventureGame,
    ) -> Option<String> {
        match command {
            c if SHOP_VERBS.contains(&c) => Some(Self::list_shop(game, args)),
            c if BUY_VERBS.contains(&c) => Some(Self::buy(game, args)),
            c if SELL_VERBS.contains(&c) => Some(Self::sell(game, args)),
            _ => None,
        }
    }
}
