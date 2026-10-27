//! The Play tab.

use super::*;

impl LoresmithStudio {
    pub(crate) fn show_play_tab(&mut self, ui: &mut egui::Ui) {
        if self.game.is_none() {
            theme::card(
                ui,
                "Ready to play",
                Some("Start a session without leaving the editor"),
                |ui| {
                    ui.label(
                        egui::RichText::new(&self.adventure.title)
                            .size(if theme::is_windows_2000() { 13.0 } else { 21.0 })
                            .strong()
                            .color(theme::text_color()),
                    );
                    let blurb = if self.adventure.description.trim().is_empty() {
                        &self.adventure.intro
                    } else {
                        &self.adventure.description
                    };
                    if !blurb.trim().is_empty() {
                        ui.label(egui::RichText::new(blurb).size(12.5).color(theme::muted()));
                    }
                    ui.add_space(10.0);
                    ui.horizontal_wrapped(|ui| {
                        theme::stat_card(
                            ui,
                            "ROOMS",
                            &self.adventure.rooms.len().to_string(),
                            theme::accent(),
                        );
                        theme::stat_card(
                            ui,
                            "ITEMS",
                            &self.adventure.items.len().to_string(),
                            theme::text_color(),
                        );
                        theme::stat_card(
                            ui,
                            "MONSTERS",
                            &self.adventure.monsters.len().to_string(),
                            theme::gold(),
                        );
                        theme::stat_card(
                            ui,
                            "QUESTS",
                            &self.adventure.quests.len().to_string(),
                            theme::success(),
                        );
                        theme::stat_card(
                            ui,
                            "SHOPS",
                            &self.adventure.shops.len().to_string(),
                            theme::muted(),
                        );
                    });
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.add(theme::primary_button("Start Game")).clicked()
                            && self.validate_adventure()
                        {
                            self.start_game();
                        }
                        if ui.add(theme::ghost_button("Validate")).clicked() {
                            self.validate_adventure();
                        }
                        if ui.add(theme::ghost_button("Open Adventure...")).clicked() {
                            self.open_adventure();
                        }
                    });
                },
            );
            ui.add_space(12.0);
            theme::card(
                ui,
                "Quick start",
                Some("The Play tab runs the adventure exactly as saved in the editor"),
                |ui| {
                    for line in [
                        "Start Game validates the document, then builds a fresh runtime session.",
                        "Use the quick command buttons or type commands in the bar at the bottom.",
                        "Save/load and autosave work inside the session; Restart begins again.",
                        "Close the session with Stop Game to return to a clean editor state.",
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("›").color(theme::accent()));
                            ui.label(egui::RichText::new(line).size(12.0).color(theme::muted()));
                        });
                    }
                },
            );
            return;
        }

        let summary = self
            .game
            .as_ref()
            .map(|game| {
                let room = game
                    .get_current_room()
                    .map(|room| room.name.clone())
                    .unwrap_or_else(|| "nowhere".to_string());
                let mut factions: Vec<_> = game.player.reputation.iter().collect();
                factions.sort_by(|left, right| left.0.cmp(right.0));
                let weapon = game
                    .player
                    .equipped_weapon
                    .and_then(|id| game.items.get(&id))
                    .map(|item| item.name.clone())
                    .unwrap_or_else(|| "Unarmed".to_string());
                let armor = game
                    .player
                    .equipped_armor
                    .and_then(|id| game.items.get(&id))
                    .map(|item| item.name.clone())
                    .unwrap_or_else(|| "No armor".to_string());
                let inventory = game
                    .player
                    .inventory
                    .iter()
                    .filter_map(|id| game.items.get(id))
                    .map(|item| item.name.clone())
                    .collect();
                let over_text = if game.game_over {
                    Some("You have been slain. Use Restart to try again.".to_string())
                } else if game.game_won {
                    Some(format!("Victory! {}", game.ending_text))
                } else {
                    None
                };
                GameSummary {
                    room,
                    health: game.player.current_health,
                    max_health: game.player.hardiness,
                    level: game.player.level,
                    xp: game.player.experience_points,
                    gold: game.player.gold,
                    carried: game.player.inventory.len(),
                    reputation: factions
                        .into_iter()
                        .map(|(faction, value)| format!("{} {:+}", faction, value))
                        .collect::<Vec<_>>()
                        .join(", "),
                    weapon,
                    armor,
                    inventory,
                    over_text,
                }
            })
            .unwrap_or_default();

        ui.horizontal_wrapped(|ui| {
            theme::stat_card(ui, "LOCATION", &summary.room, theme::accent());
            theme::stat_card(
                ui,
                "HEALTH",
                &format!("{}/{}", summary.health, summary.max_health),
                if summary.health <= summary.max_health / 3 {
                    theme::danger()
                } else {
                    theme::success()
                },
            );
            theme::stat_card(
                ui,
                "LEVEL",
                &format!(
                    "{} · {}/{} XP",
                    summary.level,
                    summary.xp,
                    summary.level * 100
                ),
                theme::text_color(),
            );
            theme::stat_card(ui, "GOLD", &summary.gold.to_string(), theme::gold());
            theme::stat_card(
                ui,
                "CARRYING",
                &format!("{} items", summary.carried),
                theme::muted(),
            );
        });

        ui.add_space(8.0);
        let health_ratio = if summary.max_health > 0 {
            summary.health as f32 / summary.max_health as f32
        } else {
            0.0
        };
        let xp_threshold = (summary.level * 100).max(1);
        let xp_ratio = (summary.xp as f32 / xp_threshold as f32).clamp(0.0, 1.0);
        ui.columns(2, |columns| {
            theme::field_panel(&mut columns[0], |ui| {
                ui.label(
                    egui::RichText::new("VITALITY")
                        .size(10.0)
                        .color(theme::muted()),
                );
                ui.add(
                    egui::ProgressBar::new(health_ratio)
                        .desired_height(12.0)
                        .text(format!("{}/{}", summary.health, summary.max_health))
                        .fill(if health_ratio <= 0.33 {
                            theme::danger()
                        } else {
                            theme::success()
                        }),
                );
            });
            theme::field_panel(&mut columns[1], |ui| {
                ui.label(
                    egui::RichText::new("EXPERIENCE")
                        .size(10.0)
                        .color(theme::muted()),
                );
                ui.add(
                    egui::ProgressBar::new(xp_ratio)
                        .desired_height(12.0)
                        .text(format!("{} / {} XP", summary.xp, xp_threshold))
                        .fill(theme::accent()),
                );
            });
        });

        if let Some(text) = &summary.over_text {
            ui.add_space(8.0);
            let color = if text.starts_with("Victory") {
                theme::success()
            } else {
                theme::danger()
            };
            egui::Frame::new()
                .fill(theme::badge_fill(color))
                .stroke(egui::Stroke::new(1.0, theme::badge_stroke(color)))
                .corner_radius(if theme::is_windows_2000() {
                    egui::CornerRadius::ZERO
                } else {
                    egui::CornerRadius::same(10)
                })
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(text).size(13.0).strong().color(color));
                });
        }

        ui.add_space(10.0);

        let content_height = (ui.available_height() - 58.0).max(200.0);
        ui.columns(2, |columns| {
            columns[0].set_min_height(content_height);
            columns[1].set_min_height(content_height);
            theme::card(
                &mut columns[0],
                "Adventure log",
                Some("Live session output"),
                |ui| {
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .auto_shrink([false, false])
                        .max_height(content_height - 54.0)
                        .show(ui, |ui| {
                            if self.game_output.is_empty() {
                                ui.label(
                                    egui::RichText::new(
                                        "No output yet. Use a quick command or type below.",
                                    )
                                    .size(12.0)
                                    .color(theme::muted()),
                                );
                            }
                            for line in &self.game_output {
                                ui.label(
                                    egui::RichText::new(line)
                                        .size(13.0)
                                        .family(egui::FontFamily::Monospace)
                                        .color(theme::text_color()),
                                );
                            }
                        });
                },
            );
            theme::card(
                &mut columns[1],
                "Character sheet",
                Some("Live runtime state"),
                |ui| {
                    ui.horizontal(|ui| {
                        theme::stat_card(ui, "WEAPON", &summary.weapon, theme::text_color());
                        theme::stat_card(ui, "ARMOR", &summary.armor, theme::text_color());
                    });
                    ui.add_space(8.0);
                    theme::field_label(ui, "INVENTORY");
                    if summary.inventory.is_empty() {
                        ui.label(
                            egui::RichText::new("Empty")
                                .size(12.0)
                                .color(theme::muted()),
                        );
                    } else {
                        for item in summary.inventory.iter().take(10) {
                            ui.label(egui::RichText::new(format!("• {}", item)).size(12.0));
                        }
                        if summary.inventory.len() > 10 {
                            ui.label(
                                egui::RichText::new(format!(
                                    "… and {} more",
                                    summary.inventory.len() - 10
                                ))
                                .size(11.0)
                                .color(theme::muted()),
                            );
                        }
                    }
                    if !summary.reputation.is_empty() {
                        ui.add_space(6.0);
                        theme::field_label(ui, "REPUTATION");
                        ui.label(
                            egui::RichText::new(&summary.reputation)
                                .size(12.0)
                                .color(theme::muted()),
                        );
                    }
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.add(theme::ghost_button("Save game")).clicked() {
                            self.game_input = "save".to_string();
                            self.send_game_command();
                        }
                        if ui.add(theme::ghost_button("Load game")).clicked() {
                            self.game_input = "load".to_string();
                            self.send_game_command();
                        }
                        if ui.add(theme::ghost_button("Stop")).clicked() {
                            self.stop_game();
                        }
                    });
                },
            );
        });

        ui.add_space(8.0);
        theme::card_panel(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("COMMAND")
                        .size(10.0)
                        .color(theme::muted()),
                );
                let input_width = (ui.available_width() - 520.0).clamp(180.0, 420.0);
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.game_input)
                        .hint_text("look, attack goblin, buy lantern, save …")
                        .desired_width(input_width),
                );
                let submitted = ui.add(theme::primary_button("Send")).clicked()
                    || (response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter)));
                if submitted {
                    self.send_game_command();
                }
                for command in ["look", "status", "inventory", "quests", "shop"] {
                    if ui.add(theme::ghost_button(command)).clicked() {
                        self.game_input = command.to_string();
                        self.send_game_command();
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(theme::ghost_button("Clear log")).clicked() {
                        self.game_output.clear();
                    }
                    if ui.add(theme::ghost_button("Restart")).clicked() {
                        self.restart_game();
                    }
                });
            });
        });
    }
}
