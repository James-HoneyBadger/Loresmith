use eframe::egui;
use sagacraft_rs::{AdventureDocument, Engine};
use sagacraft_ui as theme;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct AdventureEntry {
    title: String,
    description: String,
    path: PathBuf,
}

struct SagaFoundryPlayer {
    engine: Option<Engine>,
    adventure_path: Option<PathBuf>,
    save_path: Option<PathBuf>,
    log: Vec<String>,
    input: String,
    history: Vec<String>,
    history_index: Option<usize>,
    status: String,
    library: Vec<AdventureEntry>,
    recent: Vec<PathBuf>,
    theme_mode: theme::ThemeMode,
    theme_applied: bool,
    zoom: f32,
    show_about: bool,
    show_controls: bool,
    pending_leave: bool,
    pending_quit: bool,
    force_quit: bool,
}

impl SagaFoundryPlayer {
    fn new(cc: &eframe::CreationContext<'_>, start_path: Option<PathBuf>) -> Self {
        let mut app = Self {
            engine: None,
            adventure_path: None,
            save_path: None,
            log: Vec::new(),
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            status: "Choose an adventure from the library or open a file.".to_string(),
            library: discover_adventures(),
            recent: Vec::new(),
            theme_mode: theme::ThemeMode::Dark,
            theme_applied: false,
            zoom: 1.0,
            show_about: false,
            show_controls: false,
            pending_leave: false,
            pending_quit: false,
            force_quit: false,
        };
        if let Some(storage) = cc.storage {
            if let Some(value) = storage.get_string("theme") {
                app.theme_mode = if value.eq_ignore_ascii_case("light") {
                    theme::ThemeMode::Light
                } else {
                    theme::ThemeMode::Dark
                };
            }
            if let Some(value) = storage.get_string("recent") {
                app.recent = value
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(PathBuf::from)
                    .collect();
            }
            if let Some(value) = storage.get_string("zoom") {
                app.zoom = value.parse::<f32>().unwrap_or(1.0).clamp(0.7, 2.0);
            }
        }
        theme::set_mode(app.theme_mode);
        if let Some(path) = start_path {
            app.open_adventure(path);
        }
        app
    }

    fn open_adventure(&mut self, path: PathBuf) {
        match Engine::load(path.to_string_lossy().to_string()) {
            Ok(engine) => {
                self.log.clear();
                if !engine.intro().is_empty() {
                    self.log.push(engine.intro().to_string());
                }
                self.log.push(engine.look());
                self.status = format!("Playing {}", path.display());
                self.engine = Some(engine);
                self.adventure_path = Some(path.clone());
                self.save_path = None;
                self.remember_recent(path);
            }
            Err(error) => {
                self.status = format!("Failed to load {}: {}", path.display(), error);
            }
        }
    }

    fn open_adventure_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Saga Foundry adventures", &["json"][..])
            .add_filter("All files", &["*"][..])
            .pick_file()
        {
            self.open_adventure(path);
        }
    }

    fn load_game_dialog(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Saga Foundry saves", &["json"][..])
            .add_filter("All files", &["*"][..])
            .pick_file()
        else {
            return;
        };
        let mut engine = Engine::new("");
        match engine.load_game(&path) {
            Ok(intro) => {
                self.log.clear();
                if !intro.is_empty() {
                    self.log.push(intro);
                }
                self.log.push(engine.look());
                self.status = format!("Loaded {}", path.display());
                self.save_path = Some(path);
                self.adventure_path = None;
                self.engine = Some(engine);
            }
            Err(error) => {
                self.status = format!("Failed to load save {}: {}", path.display(), error);
            }
        }
    }

    fn save_game_dialog(&mut self) {
        let Some(engine) = &self.engine else {
            self.status = "Start an adventure before saving.".to_string();
            return;
        };
        let path = self.save_path.clone().or_else(|| {
            let name = format!(
                "{}.save.json",
                safe_file_stem(engine.document().title.as_str())
            );
            rfd::FileDialog::new()
                .add_filter("Saga Foundry saves", &["json"][..])
                .set_file_name(name)
                .save_file()
        });
        let Some(path) = path else {
            return;
        };
        match engine.save_game(&path) {
            Ok(()) => {
                self.status = format!("Saved progress to {}", path.display());
                self.save_path = Some(path);
            }
            Err(error) => {
                self.status = format!("Save failed: {}", error);
            }
        }
    }

    fn restart_game(&mut self) {
        let Some(engine) = &mut self.engine else {
            return;
        };
        match engine.restart() {
            Ok(intro) => {
                self.log.clear();
                if !intro.is_empty() {
                    self.log.push(intro);
                }
                self.log.push(engine.look());
                self.save_path = None;
                self.status = "The adventure begins again.".to_string();
            }
            Err(error) => self.status = format!("Restart failed: {}", error),
        }
    }

    fn request_library(&mut self) {
        if self.engine.is_some() {
            self.pending_leave = true;
        } else {
            self.back_to_library();
        }
    }

    fn request_quit(&mut self, ctx: &egui::Context) {
        if self.engine.is_some() {
            self.pending_quit = true;
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn back_to_library(&mut self) {
        self.engine = None;
        self.adventure_path = None;
        self.save_path = None;
        self.log.clear();
        self.input.clear();
        self.status = "Choose an adventure from the library or open a file.".to_string();
        self.pending_leave = false;
        self.pending_quit = false;
    }

    fn run_command(&mut self, command: &str) {
        let command = command.trim();
        if command.is_empty() {
            return;
        }
        self.history.push(command.to_string());
        self.history_index = None;
        self.log.push(format!("> {command}"));
        match command.to_ascii_lowercase().as_str() {
            "restart" => self.restart_game(),
            "save" => self.save_game_dialog(),
            "load" => self.load_game_dialog(),
            "quit" | "exit" => {
                self.status = "Use Game → Quit to close the player.".to_string();
            }
            _ => {
                let Some(engine) = &mut self.engine else {
                    return;
                };
                let lines = engine.send(command);
                self.log.extend(lines);
                if engine.is_over() {
                    self.status = if engine.won() {
                        "Victory! The adventure is complete.".to_string()
                    } else {
                        "Defeat. Restart or return to the library.".to_string()
                    };
                }
            }
        }
    }

    fn remember_recent(&mut self, path: PathBuf) {
        self.recent.retain(|recent| recent != &path);
        self.recent.insert(0, path);
        self.recent.truncate(8);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::O)) {
            self.open_adventure_dialog();
        }
        if ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::S)) {
            self.save_game_dialog();
        }
        if ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::L)) {
            self.load_game_dialog();
        }
        if ctx.input(|input| input.key_pressed(egui::Key::F5)) && self.engine.is_some() {
            self.restart_game();
        }
        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.show_about = false;
            self.show_controls = false;
            self.pending_leave = false;
            self.pending_quit = false;
        }
    }

    fn show_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Game", |ui| {
                if ui.button("Open Adventure…").clicked() {
                    self.open_adventure_dialog();
                    ui.close();
                }
                if ui.button("Load Saved Game…").clicked() {
                    self.load_game_dialog();
                    ui.close();
                }
                ui.separator();
                if ui.button("Save Game…").clicked() {
                    self.save_game_dialog();
                    ui.close();
                }
                if ui.button("Restart Adventure").clicked() {
                    self.restart_game();
                    ui.close();
                }
                if ui.button("Back to Library").clicked() {
                    self.request_library();
                    ui.close();
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    self.request_quit(ctx);
                    ui.close();
                }
            });
            ui.menu_button("View", |ui| {
                ui.label("Theme");
                let previous = self.theme_mode;
                ui.selectable_value(
                    &mut self.theme_mode,
                    theme::ThemeMode::Dark,
                    theme::ThemeMode::Dark.label(),
                );
                ui.selectable_value(
                    &mut self.theme_mode,
                    theme::ThemeMode::Light,
                    theme::ThemeMode::Light.label(),
                );
                if self.theme_mode != previous {
                    theme::apply(ctx, self.theme_mode);
                    self.theme_applied = true;
                    self.status = format!("{} theme applied.", self.theme_mode.label());
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Smaller Text").clicked() {
                        self.zoom = (self.zoom - 0.1).clamp(0.7, 2.0);
                    }
                    if ui.button("Larger Text").clicked() {
                        self.zoom = (self.zoom + 0.1).clamp(0.7, 2.0);
                    }
                    if ui.button("Reset").clicked() {
                        self.zoom = 1.0;
                    }
                });
                ui.separator();
                if ui.button("Toggle Fullscreen").clicked() {
                    let fullscreen =
                        ctx.input(|input| input.viewport().fullscreen.unwrap_or(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("Controls & Commands").clicked() {
                    self.show_controls = true;
                    ui.close();
                }
                if ui.button("About Saga Foundry Player").clicked() {
                    self.show_about = true;
                    ui.close();
                }
            });
        });
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        let title = self
            .engine
            .as_ref()
            .map(|engine| engine.document().title.clone())
            .unwrap_or_else(|| "Adventure library".to_string());
        let subtitle = self
            .adventure_path
            .as_ref()
            .or(self.save_path.as_ref())
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Pick a story to begin".to_string());
        let state = self.engine.as_ref().map(|engine| {
            if engine.won() {
                ("VICTORY", theme::success())
            } else if engine.is_over() {
                ("DEFEATED", theme::danger())
            } else {
                ("IN PLAY", theme::accent())
            }
        });
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.add_space(7.0);
                ui.label(
                    egui::RichText::new("Saga Foundry")
                        .size(25.0)
                        .strong()
                        .color(theme::accent()),
                );
                ui.label(
                    egui::RichText::new("PLAYER")
                        .size(10.5)
                        .strong()
                        .color(theme::muted()),
                );
            });
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(12.0);
            ui.vertical(|ui| {
                ui.add_space(9.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(title)
                            .size(14.0)
                            .strong()
                            .color(theme::text_color()),
                    );
                    if let Some((label, color)) = state {
                        theme::badge(ui, label, color);
                    }
                });
                ui.label(
                    egui::RichText::new(subtitle)
                        .size(10.5)
                        .color(theme::muted()),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                if self.engine.is_some() {
                    if ui.add(theme::ghost_button("Library")).clicked() {
                        self.request_library();
                    }
                    if ui.add(theme::ghost_button("Restart")).clicked() {
                        self.restart_game();
                    }
                    if ui.add(theme::ghost_button("Load")).clicked() {
                        self.load_game_dialog();
                    }
                    if ui.add(theme::primary_button("Save")).clicked() {
                        self.save_game_dialog();
                    }
                } else if ui.add(theme::primary_button("Open Adventure")).clicked() {
                    self.open_adventure_dialog();
                }
            });
        });
    }

    fn show_landing(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                theme::card_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Your next story starts here")
                                    .size(30.0)
                                    .strong()
                                    .color(theme::text_color()),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(
                                    "A focused, standalone player for Saga Foundry adventures. No editor, no clutter — just the story, your character, and the choices ahead.",
                                )
                                .size(13.0)
                                .color(theme::muted()),
                            );
                            ui.add_space(14.0);
                            ui.horizontal(|ui| {
                                if ui.add(theme::primary_button("Open Adventure")).clicked() {
                                    self.open_adventure_dialog();
                                }
                                if ui.add(theme::ghost_button("Load Saved Game")).clicked() {
                                    self.load_game_dialog();
                                }
                                if ui.add(theme::ghost_button("Controls")).clicked() {
                                    self.show_controls = true;
                                }
                            });
                            ui.add_space(10.0);
                            ui.horizontal_wrapped(|ui| {
                                theme::stat_card(
                                    ui,
                                    "LIBRARY",
                                    &self.library.len().to_string(),
                                    theme::accent(),
                                );
                                theme::stat_card(
                                    ui,
                                    "RECENT",
                                    &self.recent.len().to_string(),
                                    theme::gold(),
                                );
                                ui.label(
                                    egui::RichText::new(
                                        "save anywhere · keyboard friendly · no editor clutter",
                                    )
                                    .size(11.0)
                                    .color(theme::muted()),
                                );
                            });
                        });
                    });
                });

                ui.add_space(14.0);
                theme::section_heading(
                    ui,
                    "Adventure library",
                    Some("Bundled demos and adventures found beside the player"),
                );
                ui.add_space(4.0);
                if self.library.is_empty() {
                    theme::card(
                        ui,
                        "No adventures found",
                        Some("Open any Saga Foundry JSON adventure"),
                        |ui| {
                            ui.label(
                                egui::RichText::new(
                                    "Use Open Adventure to choose an adventure file. You can also run the player with a path: sagacraft_play my_adventure.json",
                                )
                                .size(12.0)
                                .color(theme::muted()),
                            );
                        },
                    );
                }
                for entry in &self.library {
                    let mut play = false;
                    egui::Frame::new()
                        .fill(theme::surface())
                        .stroke(egui::Stroke::new(1.0, theme::border()))
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::symmetric(14, 11))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&entry.title)
                                            .size(16.0)
                                            .strong()
                                            .color(theme::text_color()),
                                    );
                                    ui.label(
                                        egui::RichText::new(&entry.description)
                                            .size(11.5)
                                            .color(theme::muted()),
                                    );
                                    ui.label(
                                        egui::RichText::new(entry.path.display().to_string())
                                            .size(10.0)
                                            .color(theme::muted()),
                                    );
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.add(theme::primary_button("Play")).clicked() {
                                            play = true;
                                        }
                                    },
                                );
                            });
                        });
                    if play {
                        self.open_adventure(entry.path.clone());
                        break;
                    }
                    ui.add_space(6.0);
                }

                if !self.recent.is_empty() {
                    ui.add_space(14.0);
                    theme::section_heading(ui, "Recently played", None);
                    ui.add_space(4.0);
                    theme::card_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let mut open = None;
                        for path in &self.recent {
                            if ui
                                .add(egui::Button::new(path.display().to_string()).selected(false))
                                .clicked()
                            {
                                open = Some(path.clone());
                            }
                        }
                        if let Some(path) = open {
                            self.open_adventure(path);
                        }
                    });
                }
            });
    }

    fn show_game(&mut self, ui: &mut egui::Ui) {
        let Some(engine) = &self.engine else {
            return;
        };
        let room_name = engine
            .game
            .get_current_room()
            .map(|room| room.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let exits = engine
            .game
            .get_current_room()
            .map(|room| {
                let mut exits: Vec<_> = room.exits.keys().cloned().collect();
                exits.sort();
                exits.join(" · ")
            })
            .unwrap_or_default();
        let health = engine.game.player.current_health;
        let max_health = engine.game.player.hardiness.max(1);
        let level = engine.game.player.level;
        let xp = engine.game.player.experience_points;
        let xp_threshold = (level * 100).max(1);
        let gold = engine.game.player.gold;
        let inventory: Vec<String> = engine
            .game
            .player
            .inventory
            .iter()
            .filter_map(|id| engine.game.items.get(id))
            .map(|item| item.name.clone())
            .collect();
        let weapon = engine
            .game
            .player
            .equipped_weapon
            .and_then(|id| engine.game.items.get(&id))
            .map(|item| item.name.clone())
            .unwrap_or_else(|| "Unarmed".to_string());
        let armor = engine
            .game
            .player
            .equipped_armor
            .and_then(|id| engine.game.items.get(&id))
            .map(|item| item.name.clone())
            .unwrap_or_else(|| "None".to_string());
        let reputation = {
            let mut factions: Vec<_> = engine.game.player.reputation.iter().collect();
            factions.sort_by(|left, right| left.0.cmp(right.0));
            factions
                .into_iter()
                .map(|(faction, value)| format!("{} {:+}", faction, value))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let ending = engine.ending().map(|text| text.to_string());
        let won = engine.won();

        let full_size = ui.available_size();
        let sidebar_width = 320.0;
        let gap = 12.0;
        let main_width = (full_size.x - sidebar_width - gap).max(420.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(main_width, full_size.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    if let Some(text) = &ending {
                        let color = if won {
                            theme::success()
                        } else {
                            theme::danger()
                        };
                        egui::Frame::new()
                            .fill(theme::badge_fill(color))
                            .stroke(egui::Stroke::new(1.0, theme::badge_stroke(color)))
                            .corner_radius(egui::CornerRadius::same(10))
                            .inner_margin(egui::Margin::symmetric(14, 10))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    egui::RichText::new(if won { "Victory" } else { "Defeat" })
                                        .size(16.0)
                                        .strong()
                                        .color(color),
                                );
                                ui.label(
                                    egui::RichText::new(text)
                                        .size(12.5)
                                        .color(theme::text_color()),
                                );
                            });
                        ui.add_space(8.0);
                    }
                    let log_height = (ui.available_height() - 58.0).max(220.0);
                    theme::card(ui, "Story", Some("Live adventure log"), |ui| {
                        egui::ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .auto_shrink([false, false])
                            .max_height(log_height)
                            .show(ui, |ui| {
                                for line in &self.log {
                                    if line.trim().is_empty() {
                                        ui.add_space(5.0);
                                    } else {
                                        ui.label(
                                            egui::RichText::new(line)
                                                .size(14.0)
                                                .color(theme::text_color()),
                                        );
                                    }
                                }
                            });
                    });
                },
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                egui::vec2(sidebar_width, full_size.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    theme::card(ui, "Vitals", Some("Your current condition"), |ui| {
                        ui.horizontal(|ui| {
                            theme::stat_card(
                                ui,
                                "HEALTH",
                                &format!("{health}/{max_health}"),
                                theme::success(),
                            );
                            theme::stat_card(ui, "GOLD", &gold.to_string(), theme::gold());
                        });
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(format!("Level {level} · {xp}/{xp_threshold} XP"))
                                .size(11.0)
                                .color(theme::muted()),
                        );
                        ui.add(
                            egui::ProgressBar::new(xp as f32 / xp_threshold as f32)
                                .desired_height(8.0)
                                .fill(theme::accent()),
                        );
                    });
                    ui.add_space(8.0);
                    theme::card(ui, "Location", None, |ui| {
                        ui.label(
                            egui::RichText::new(room_name)
                                .size(16.0)
                                .strong()
                                .color(theme::accent()),
                        );
                        if !exits.is_empty() {
                            ui.label(
                                egui::RichText::new(format!("Exits: {exits}"))
                                    .size(11.0)
                                    .color(theme::muted()),
                            );
                        }
                    });
                    ui.add_space(8.0);
                    theme::card(ui, "Character", None, |ui| {
                        theme::field_label(ui, "WIELDING");
                        ui.label(egui::RichText::new(weapon).size(12.5));
                        theme::field_label(ui, "WEARING");
                        ui.label(egui::RichText::new(armor).size(12.5));
                        theme::field_label(ui, "INVENTORY");
                        if inventory.is_empty() {
                            ui.label(
                                egui::RichText::new("Empty")
                                    .size(11.5)
                                    .color(theme::muted()),
                            );
                        } else {
                            for item in inventory.iter().take(8) {
                                ui.label(egui::RichText::new(format!("• {item}")).size(11.5));
                            }
                            if inventory.len() > 8 {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "… and {} more",
                                        inventory.len() - 8
                                    ))
                                    .size(11.0)
                                    .color(theme::muted()),
                                );
                            }
                        }
                        if !reputation.is_empty() {
                            theme::field_label(ui, "REPUTATION");
                            ui.label(
                                egui::RichText::new(reputation)
                                    .size(11.0)
                                    .color(theme::muted()),
                            );
                        }
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.add(theme::ghost_button("Journal")).clicked() {
                            self.run_command("quests");
                        }
                        if ui.add(theme::ghost_button("Status")).clicked() {
                            self.run_command("status");
                        }
                    });
                },
            );
        });
    }

    fn show_command_bar(&mut self, ui: &mut egui::Ui) {
        theme::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("COMMAND")
                        .size(10.0)
                        .color(theme::muted()),
                );
                let width = (ui.available_width() - 520.0).clamp(160.0, 440.0);
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .hint_text("look, attack goblin, buy lantern, save …")
                        .desired_width(width),
                );
                let submitted = ui.add(theme::primary_button("Send")).clicked()
                    || (response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter)));
                if submitted {
                    let command = self.input.clone();
                    self.input.clear();
                    self.run_command(&command);
                }
                for command in ["look", "status", "inventory", "quests", "help"] {
                    if ui.add(theme::ghost_button(command)).clicked() {
                        self.run_command(command);
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.engine.as_ref().is_some_and(|engine| engine.is_over())
                        && ui.add(theme::primary_button("Restart")).clicked()
                    {
                        self.restart_game();
                    }
                });
            });
        });
    }

    fn show_windows(&mut self, ctx: &egui::Context) {
        if self.pending_leave {
            egui::Window::new("Leave this adventure?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Unsaved progress will be lost. You can save first from the header.");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.add(theme::primary_button("Leave")).clicked() {
                            self.back_to_library();
                        }
                        if ui.add(theme::ghost_button("Cancel")).clicked() {
                            self.pending_leave = false;
                        }
                    });
                });
        }
        if self.pending_quit {
            egui::Window::new("Quit Saga Foundry Player?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Unsaved progress will be lost.");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.add(theme::primary_button("Quit")).clicked() {
                            self.force_quit = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.add(theme::ghost_button("Cancel")).clicked() {
                            self.pending_quit = false;
                        }
                    });
                });
        }
        if self.show_about {
            let mut open = self.show_about;
            egui::Window::new("About Saga Foundry Player")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("Saga Foundry Player")
                            .size(22.0)
                            .strong()
                            .color(theme::accent()),
                    );
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.add_space(8.0);
                    ui.label(
                        "A standalone game player for Saga Foundry adventures, built with egui and the Saga Foundry engine.",
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Part of the Saga Foundry suite: Engine · Studio · Player · Console · Workbench")
                            .size(11.0)
                            .color(theme::muted()),
                    );
                });
            self.show_about = open;
        }
        if self.show_controls {
            let mut open = self.show_controls;
            egui::Window::new("Controls & Commands")
                .open(&mut open)
                .default_size([620.0, 560.0])
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("Keyboard")
                            .strong()
                            .color(theme::text_color()),
                    );
                    for (key, action) in [
                        ("Enter", "Send a command"),
                        ("↑ / ↓", "Browse command history"),
                        ("Ctrl+O", "Open an adventure"),
                        ("Ctrl+S", "Save progress"),
                        ("Ctrl+L", "Load a save"),
                        ("F5", "Restart the adventure"),
                        ("Esc", "Close dialogs"),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(key)
                                    .monospace()
                                    .size(12.0)
                                    .color(theme::accent()),
                            );
                            ui.label(egui::RichText::new(action).size(12.0));
                        });
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(420.0)
                        .show(ui, |ui| {
                            let mut help = sagacraft_rs::commands::help_text();
                            ui.add(
                                egui::TextEdit::multiline(&mut help)
                                    .font(egui::TextStyle::Monospace)
                                    .interactive(false)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                });
            self.show_controls = open;
        }
    }

    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(&self.status)
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                if self.engine.is_some() {
                    ui.label(
                        egui::RichText::new("Enter send · ↑↓ history · Ctrl+S save")
                            .size(10.5)
                            .color(theme::muted()),
                    );
                }
                if !self.library.is_empty() {
                    theme::badge(
                        ui,
                        &format!("{} ADVENTURES", self.library.len()),
                        theme::accent(),
                    );
                }
            });
        });
    }

    fn handle_history_keys(&mut self, ui: &egui::Ui) {
        if self.history.is_empty() {
            return;
        }
        let up = ui.input(|input| input.key_pressed(egui::Key::ArrowUp));
        let down = ui.input(|input| input.key_pressed(egui::Key::ArrowDown));
        if up {
            let next = match self.history_index {
                Some(0) => 0,
                Some(index) => index - 1,
                None => self.history.len() - 1,
            };
            self.history_index = Some(next);
            self.input = self.history[next].clone();
        } else if down {
            match self.history_index {
                Some(index) if index + 1 < self.history.len() => {
                    self.history_index = Some(index + 1);
                    self.input = self.history[index + 1].clone();
                }
                Some(_) => {
                    self.history_index = None;
                    self.input.clear();
                }
                None => {}
            }
        }
    }
}

impl eframe::App for SagaFoundryPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(
            "theme",
            match self.theme_mode {
                theme::ThemeMode::Dark => "dark".to_string(),
                theme::ThemeMode::Light => "light".to_string(),
            },
        );
        storage.set_string(
            "recent",
            self.recent
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        );
        storage.set_string("zoom", self.zoom.to_string());
    }

    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root_ui.ctx().clone();
        if !self.theme_applied || theme::mode() != self.theme_mode {
            theme::apply(&ctx, self.theme_mode);
            self.theme_applied = true;
        }
        ctx.set_zoom_factor(self.zoom);
        if ctx.input(|input| input.viewport().close_requested())
            && self.engine.is_some()
            && !self.force_quit
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending_quit = true;
        }
        self.handle_shortcuts(&ctx);

        egui::Panel::top("player_menu")
            .exact_size(25.0)
            .show(root_ui, |ui| self.show_menu_bar(&ctx, ui));
        egui::Panel::top("player_header")
            .exact_size(70.0)
            .show(root_ui, |ui| self.show_header(ui));
        egui::Panel::bottom("player_status")
            .exact_size(26.0)
            .show(root_ui, |ui| self.show_status_bar(ui));
        if self.engine.is_some() {
            egui::Panel::bottom("player_command")
                .exact_size(58.0)
                .show(root_ui, |ui| self.show_command_bar(ui));
        }
        egui::CentralPanel::default()
            .frame(theme::content_frame())
            .show(root_ui, |ui| {
                if self.engine.is_some() {
                    self.show_game(ui);
                } else {
                    self.show_landing(ui);
                }
            });

        self.handle_history_keys(root_ui);
        self.show_windows(&ctx);
    }
}

fn discover_adventures() -> Vec<AdventureEntry> {
    let mut roots = vec![
        PathBuf::from("examples/adventures"),
        PathBuf::from("adventures"),
    ];
    if let Ok(executable) = std::env::current_exe()
        && let Some(directory) = executable.parent()
    {
        for ancestor in directory.ancestors().take(5) {
            roots.push(ancestor.join("examples/adventures"));
            roots.push(ancestor.join("adventures"));
            roots.push(ancestor.join("share/sagacraft/adventures"));
        }
    }

    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    for root in roots {
        let Ok(read_dir) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            if !seen.insert(canonical) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(document) = AdventureDocument::from_json_str(&text) else {
                continue;
            };
            let description = if document.description.trim().is_empty() {
                first_sentence(&document.intro)
            } else {
                document.description.clone()
            };
            let title = if document.title.trim().is_empty() {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("Untitled adventure")
                    .replace(['_', '-'], " ")
            } else {
                document.title
            };
            entries.push(AdventureEntry {
                title,
                description: truncate(&description, 190),
                path,
            });
        }
    }
    entries.sort_by(|left, right| left.title.cmp(&right.title));
    entries.dedup_by(|left, right| left.path == right.path);
    entries
}

fn first_sentence(text: &str) -> String {
    let trimmed = text.trim();
    let end = trimmed.find(['.', '!', '?']).map(|index| index + 1);
    truncate(end.map_or(trimmed, |index| &trimmed[..index]), 190)
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        text.to_string()
    } else {
        let mut shortened: String = text.chars().take(limit.saturating_sub(1)).collect();
        shortened.push('…');
        shortened
    }
}

fn safe_file_stem(title: &str) -> String {
    let stem: String = title
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let stem = stem.trim_matches('-').to_string();
    if stem.is_empty() {
        "adventure".to_string()
    } else {
        stem
    }
}

fn parse_args() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    let mut path = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("Saga Foundry Player");
                println!("Usage: sagacraft_play [adventure.json]");
                println!();
                println!("  -a, --adventure <path>  Adventure to open");
                println!("  -h, --help              Show this help");
                println!("  -V, --version           Show the version");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!(
                    "Saga Foundry Player {} (sagacraft_play)",
                    env!("CARGO_PKG_VERSION")
                );
                std::process::exit(0);
            }
            "-a" | "--adventure" => match args.next() {
                Some(value) => path = Some(PathBuf::from(value)),
                None => {
                    eprintln!("--adventure requires a path.");
                    std::process::exit(2);
                }
            },
            other if !other.starts_with('-') => path = Some(PathBuf::from(other)),
            unknown => {
                eprintln!("Unknown argument: {unknown}");
                std::process::exit(2);
            }
        }
    }
    path
}

fn main() -> eframe::Result<()> {
    let start_path = parse_args();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Saga Foundry Player",
        options,
        Box::new(move |cc| Ok(Box::new(SagaFoundryPlayer::new(cc, start_path.clone())))),
    )
}
