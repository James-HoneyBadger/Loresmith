use eframe::egui;
use loresmith_rs::{
    AdventureDocument, AdventureGame, BasicWorldSystem, CombatSystem, DialogueDef, EndingDef,
    ExitDef, InventoryRef, InventorySystem, ItemDef, ItemType, MonsterDef, MonsterStatus,
    ObjectiveDef, ObjectiveKind, QuestDef, QuestRewardsDef, QuestSystem, RewardItem, RoomDef,
    ShopDef, ShopStockDef, ShopSystem, StageDef, StructuredObjective,
};
use loresmith_ui as theme;
use std::path::PathBuf;

/// The GUI edits the canonical document directly so every field round-trips.
type AdventureData = AdventureDocument;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Loresmith Studio",
        options,
        Box::new(|cc| Ok(Box::new(LoresmithStudio::new(cc)))),
    )
}

struct LoresmithStudio {
    adventure: AdventureData,
    current_file: Option<PathBuf>,
    modified: bool,
    active_tab: Tab,
    status: String,
    // Tab-specific state
    selected_room: Option<usize>,
    selected_item: Option<usize>,
    selected_monster: Option<usize>,
    selected_quest: Option<usize>,
    selected_shop: Option<usize>,
    room_filter: String,
    item_filter: String,
    monster_filter: String,
    quest_filter: String,
    shop_filter: String,
    // Play tab state
    game: Option<AdventureGame>,
    game_output: Vec<String>,
    game_input: String,
    // Exit confirmation
    show_exit_confirm: bool,
    // Add-exit dialog state
    new_exit_direction: String,
    new_exit_target: i32,
    // JSON preview / diff support
    last_saved_json: Option<String>,
    diff_text: Option<String>,
    // Undo / redo snapshots
    undo_stack: Vec<AdventureData>,
    redo_stack: Vec<AdventureData>,
    frame_snapshot: Option<AdventureData>,
    theme_mode: theme::ThemeMode,
    theme_applied: bool,
    classic_chrome: bool,
}

impl Default for LoresmithStudio {
    fn default() -> Self {
        let adventure = starter_document();
        let last_saved_json = adventure.to_json_string().ok();
        Self {
            adventure,
            current_file: None,
            modified: false,
            active_tab: Tab::Play,
            status: "Ready. Open an adventure or start editing.".to_string(),
            selected_room: Some(0),
            selected_item: Some(0),
            selected_monster: Some(0),
            selected_quest: Some(0),
            selected_shop: None,
            room_filter: String::new(),
            item_filter: String::new(),
            monster_filter: String::new(),
            quest_filter: String::new(),
            shop_filter: String::new(),
            game: None,
            game_output: Vec::new(),
            game_input: String::new(),
            show_exit_confirm: false,
            new_exit_direction: "north".to_string(),
            new_exit_target: 1,
            last_saved_json,
            diff_text: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            frame_snapshot: None,
            theme_mode: theme::ThemeMode::Dark,
            theme_applied: false,
            classic_chrome: false,
        }
    }
}

impl LoresmithStudio {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        if let Some(storage) = cc.storage
            && let Some(mode) = storage.get_string("theme")
        {
            app.theme_mode = theme::ThemeMode::from_key(&mode);
        }
        theme::set_mode(app.theme_mode);
        app
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Tab {
    #[default]
    Play,
    Info,
    Rooms,
    Items,
    Monsters,
    Quests,
    Shops,
    Preview,
}

fn starter_document() -> AdventureDocument {
    let mut document = AdventureDocument {
        id: "new_adventure".to_string(),
        title: "New Adventure".to_string(),
        description: "A new Loresmith adventure.".to_string(),
        intro: "Welcome to your new adventure!".to_string(),
        start_room: 1,
        ..AdventureDocument::default()
    };
    document.rooms.push(RoomDef {
        id: 1,
        name: "Starting Room".to_string(),
        description: "A simple room to begin your adventure.".to_string(),
        ..RoomDef::default()
    });
    document.items.push(ItemDef {
        id: 1,
        name: "Brass Lantern".to_string(),
        description: "A shiny brass lantern that provides light.".to_string(),
        item_type: ItemType::Normal,
        value: 25,
        weight: 2,
        is_light_source: true,
        location: 1,
        ..ItemDef::default()
    });
    document.monsters.push(MonsterDef {
        id: 1,
        name: "Goblin".to_string(),
        description: "A small green goblin with a club.".to_string(),
        hardiness: 8,
        agility: 12,
        friendliness: MonsterStatus::Friendly,
        gold: 5,
        room_id: 1,
        ..MonsterDef::default()
    });
    document.quests.push(QuestDef {
        id: 1,
        title: "Light the Path".to_string(),
        description: "Secure a light source and reach the Shadow Gallery.".to_string(),
        objectives: vec![
            ObjectiveDef::Text("Pick up the brass lantern".to_string()),
            ObjectiveDef::Text("Enter the Shadow Gallery with light".to_string()),
        ],
        rewards: QuestRewardsDef {
            gold: 40,
            xp: 60,
            ..QuestRewardsDef::default()
        },
        ..QuestDef::default()
    });
    document
}

fn ensure_structured_objective(
    objective: &mut ObjectiveDef,
    index: usize,
) -> &mut StructuredObjective {
    if let ObjectiveDef::Text(text) = objective {
        let text = text.clone();
        *objective =
            ObjectiveDef::Structured(loresmith_rs::document::parse_text_objective(index, &text));
    }
    match objective {
        ObjectiveDef::Structured(objective) => objective,
        ObjectiveDef::Text(_) => unreachable!("text objectives are converted above"),
    }
}

fn objective_kind_label(kind: ObjectiveKind) -> &'static str {
    match kind {
        ObjectiveKind::KillMonster => "Kill monster",
        ObjectiveKind::CollectItem => "Collect item",
        ObjectiveKind::ReachRoom => "Reach room",
        ObjectiveKind::TalkToNpc => "Talk to NPC",
        ObjectiveKind::Defend => "Defend",
        ObjectiveKind::Deliver => "Deliver",
        ObjectiveKind::Discover => "Discover",
        ObjectiveKind::Puzzle => "Puzzle",
        ObjectiveKind::Other => "Other",
    }
}

#[derive(Default)]
struct GameSummary {
    room: String,
    health: i32,
    max_health: i32,
    level: i32,
    xp: i32,
    gold: i32,
    carried: usize,
    reputation: String,
    weapon: String,
    armor: String,
    inventory: Vec<String>,
    over_text: Option<String>,
}

impl eframe::App for LoresmithStudio {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("theme", self.theme_mode.key().to_string());
    }

    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx_owned = root_ui.ctx().clone();
        let ctx = &ctx_owned;
        if !self.modified && self.frame_snapshot.is_none() {
            self.frame_snapshot = Some(self.adventure.clone());
        }
        if ctx.input(|input| input.viewport().close_requested()) && self.modified {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_exit_confirm = true;
        }

        let save_pressed =
            ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::S));
        if save_pressed {
            if ctx.input(|input| input.modifiers.shift) {
                self.save_adventure_as();
            } else {
                self.save_adventure();
            }
        }
        if ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::O)) {
            self.open_adventure();
        }
        if ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::N)) {
            self.new_adventure();
        }
        if ctx.input(|input| input.key_pressed(egui::Key::F5)) && self.validate_adventure() {
            self.start_game();
            self.active_tab = Tab::Play;
        }
        if ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::Z)) {
            self.undo();
        }
        if ctx.input(|input| {
            input.modifiers.command && input.modifiers.shift && input.key_pressed(egui::Key::Z)
        }) {
            self.redo();
        }

        if !self.theme_applied || theme::mode() != self.theme_mode {
            theme::apply(ctx, self.theme_mode);
            self.theme_applied = true;
        }
        let classic_chrome = theme::is_windows_2000();
        if classic_chrome != self.classic_chrome {
            self.classic_chrome = classic_chrome;
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(!classic_chrome));
        }

        if theme::is_windows_2000() {
            egui::Panel::top("app_caption")
                .exact_size(22.0)
                .frame(egui::Frame::NONE)
                .show(root_ui, |ui| {
                    self.show_caption_bar(ctx, ui);
                });
            egui::Panel::top("menu_bar")
                .exact_size(23.0)
                .show(root_ui, |ui| {
                    self.show_menu_bar(ctx, ui);
                });
            egui::Panel::top("app_toolbar")
                .exact_size(32.0)
                .show(root_ui, |ui| {
                    self.show_toolbar(ui);
                });
        } else {
            egui::Panel::top("menu_bar")
                .exact_size(25.0)
                .show(root_ui, |ui| {
                    self.show_menu_bar(ctx, ui);
                });

            egui::Panel::top("app_header")
                .exact_size(70.0)
                .show(root_ui, |ui| {
                    self.show_header(ui);
                });
        }

        egui::Panel::bottom("status_bar")
            .exact_size(30.0)
            .show(root_ui, |ui| {
                self.show_status_bar(ui);
            });

        egui::Panel::left("nav_rail")
            .exact_size(198.0)
            .show(root_ui, |ui| {
                self.show_nav(ui);
            });

        egui::CentralPanel::default()
            .frame(theme::content_frame())
            .show(root_ui, |ui| {
                self.show_main_ui(ui);
            });

        // Exit confirmation dialog
        if self.show_exit_confirm {
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("You have unsaved changes. Are you sure you want to exit?");
                    ui.horizontal(|ui| {
                        if ui.button("Save & Exit").clicked() && self.save_adventure() {
                            self.show_exit_confirm = false;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Exit Without Saving").clicked() {
                            self.modified = false;
                            self.show_exit_confirm = false;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_exit_confirm = false;
                        }
                    });
                });
        }

        theme::window_resize_handles(root_ui);

        if let Some(diff) = self.diff_text.clone() {
            let mut open = true;
            egui::Window::new("JSON Diff (saved → current)")
                .open(&mut open)
                .default_size([800.0, 500.0])
                .show(ctx, |ui| {
                    let mut text = diff.as_str();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut text)
                                .font(egui::TextStyle::Monospace)
                                .interactive(false),
                        );
                    });
                });
            if !open {
                self.diff_text = None;
            }
        }
    }
}

impl LoresmithStudio {
    fn show_caption_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let file_name = self
            .current_file
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled adventure".to_string());
        let caption = format!(
            "Loresmith Studio - {}{}",
            file_name,
            if self.modified { " *" } else { "" }
        );
        let maximized = ctx.input(|input| input.viewport().maximized.unwrap_or(false));
        let Some(action) = theme::caption_bar(ui, &caption, maximized) else {
            return;
        };
        match action {
            theme::CaptionAction::Minimize => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
            theme::CaptionAction::Maximize => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }
            theme::CaptionAction::Close => {
                if self.modified {
                    self.show_exit_confirm = true;
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        theme::toolbar(ui, |ui| {
            if ui.add(theme::ghost_button("New")).clicked() {
                self.new_adventure();
            }
            if ui.add(theme::ghost_button("Open")).clicked() {
                self.open_adventure();
            }
            if ui.add(theme::ghost_button("Save")).clicked() {
                self.save_adventure();
            }
            if ui.add(theme::ghost_button("Save As")).clicked() {
                self.save_adventure_as();
            }
            ui.separator();
            if ui.add(theme::ghost_button("Validate")).clicked() {
                self.validate_adventure();
            }
            if ui.add(theme::primary_button("Play")).clicked() && self.validate_adventure() {
                self.start_game();
                self.active_tab = Tab::Play;
            }
            ui.separator();
            if ui.add(theme::ghost_button("Undo")).clicked() {
                self.undo();
            }
            if ui.add(theme::ghost_button("Redo")).clicked() {
                self.redo();
            }
        });
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Loresmith")
                        .size(25.0)
                        .strong()
                        .color(theme::accent()),
                );
                ui.label(
                    egui::RichText::new("STUDIO")
                        .size(11.0)
                        .color(theme::muted()),
                );
            });

            ui.add_space(24.0);
            ui.separator();
            ui.add_space(12.0);
            ui.vertical(|ui| {
                ui.add_space(10.0);
                let file_name = self
                    .current_file
                    .as_ref()
                    .and_then(|path| path.file_name())
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Untitled adventure".to_string());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(file_name)
                            .size(14.0)
                            .strong()
                            .color(theme::text_color()),
                    );
                    if self.modified {
                        theme::badge(ui, "UNSAVED", theme::gold());
                    } else {
                        theme::badge(ui, "SAVED", theme::success());
                    }
                });
                let path_text = self
                    .current_file
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "No file selected".to_string());
                ui.label(
                    egui::RichText::new(path_text)
                        .size(10.5)
                        .color(theme::muted()),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                if ui.add(theme::primary_button("Play")).clicked() && self.validate_adventure() {
                    self.start_game();
                    self.active_tab = Tab::Play;
                }
                if ui.add(theme::ghost_button("Save")).clicked() {
                    self.save_adventure();
                }
                if ui.add(theme::ghost_button("Validate")).clicked() {
                    self.validate_adventure();
                }
            });
        });
    }

    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        theme::status_panel(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                let color = if self.status.starts_with("Valid") || self.status.contains("Saved") {
                    theme::success()
                } else if self.status.starts_with("Error")
                    || self.status.contains("failed")
                    || self.status.contains("Validation errors")
                {
                    theme::danger()
                } else {
                    theme::accent()
                };
                ui.label(egui::RichText::new("●").color(color).size(11.0));
                ui.label(
                    egui::RichText::new(&self.status)
                        .size(11.5)
                        .color(theme::text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if self.modified {
                        theme::badge(ui, "UNSAVED", theme::gold());
                    } else {
                        theme::badge(ui, "SAVED", theme::success());
                    }
                    ui.label(
                        egui::RichText::new("Ctrl+S save  ·  F5 play  ·  Ctrl+Z undo")
                            .size(10.5)
                            .color(theme::muted()),
                    );
                });
            });
        });
    }

    fn show_nav(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("WORKSPACE")
                    .size(9.5)
                    .strong()
                    .color(theme::muted()),
            );
        });
        ui.add_space(4.0);
        for (tab, title, subtitle) in [
            (Tab::Play, "Play", "Run the adventure"),
            (Tab::Info, "Overview", "Identity and ending"),
            (Tab::Rooms, "Rooms", "Maps, exits and locks"),
            (Tab::Items, "Items", "Gear, loot and light"),
            (Tab::Monsters, "Monsters", "NPCs, dialogue and loot"),
            (Tab::Quests, "Quests", "Goals, stages and rewards"),
            (Tab::Shops, "Shops", "Stock and pricing"),
            (Tab::Preview, "Preview", "JSON and validation"),
        ] {
            if theme::nav_button(ui, self.active_tab == tab, title, subtitle) {
                self.active_tab = tab;
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("CONTENT")
                    .size(9.5)
                    .strong()
                    .color(theme::muted()),
            );
        });
        ui.add_space(4.0);
        theme::raised_panel(ui, |ui| {
            for (label, value) in [
                ("Rooms", self.adventure.rooms.len()),
                ("Items", self.adventure.items.len()),
                ("Monsters", self.adventure.monsters.len()),
                ("Quests", self.adventure.quests.len()),
                ("Shops", self.adventure.shops.len()),
            ] {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(label).size(11.0).color(theme::muted()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(value.to_string())
                                .size(12.0)
                                .strong()
                                .color(theme::text_color()),
                        );
                    });
                });
            }
        });

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(
                    egui::RichText::new("Loresmith 4.1.0")
                        .size(10.0)
                        .color(theme::muted()),
                );
            });
        });
    }

    fn show_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        theme::menu_background(ui);
        egui::MenuBar::new()
            .style(theme::menu_style)
            .config(egui::containers::menu::MenuConfig::new().style(theme::menu_style))
            .ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Adventure").clicked() {
                        self.new_adventure();
                        ui.close();
                    }
                    if ui.button("Open Adventure...").clicked() {
                        self.open_adventure();
                        ui.close();
                    }
                    if ui.button("Save Adventure").clicked() {
                        self.save_adventure();
                        ui.close();
                    }
                    if ui.button("Save Adventure As...").clicked() {
                        self.save_adventure_as();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        if self.modified {
                            self.show_exit_confirm = true;
                        } else {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                    ui.separator();
                    if ui.button("Undo").clicked() {
                        self.undo();
                        ui.close();
                    }
                    if ui.button("Redo").clicked() {
                        self.redo();
                        ui.close();
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if ui.button("Validate Adventure").clicked() {
                        self.validate_adventure();
                        ui.close();
                    }
                    if ui.button("Export to JSON").clicked() {
                        self.export_to_json();
                        ui.close();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        self.undo();
                        ui.close();
                    }
                    if ui.button("Redo").clicked() {
                        self.redo();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Copy JSON to Clipboard").clicked() {
                        self.copy_json_to_clipboard(ui.ctx());
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.label("Theme");
                    let previous = self.theme_mode;
                    for mode in theme::ThemeMode::ALL {
                        ui.selectable_value(&mut self.theme_mode, mode, mode.label());
                    }
                    if self.theme_mode != previous {
                        theme::apply(ctx, self.theme_mode);
                        self.theme_applied = true;
                        self.status = format!("{} theme applied.", self.theme_mode.label());
                        ui.close();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About Loresmith Studio").clicked() {
                        self.show_about();
                        ui.close();
                    }
                });
            });
    }

    fn show_main_ui(&mut self, ui: &mut egui::Ui) {
        let (title, subtitle) = match self.active_tab {
            Tab::Play => (
                "Play",
                "Run and test the adventure from the current editor state",
            ),
            Tab::Info => (
                "Overview",
                "Adventure identity, presentation, starting state and ending",
            ),
            Tab::Rooms => ("Rooms", "Build the map, connect exits and place items"),
            Tab::Items => (
                "Items",
                "Weapons, armor, consumables, light sources and treasure",
            ),
            Tab::Monsters => ("Monsters", "NPCs and enemies, dialogue topics and loot"),
            Tab::Quests => ("Quests", "Staged objectives, rewards and turn-in rules"),
            Tab::Shops => ("Shops", "Stock, gold and pricing for merchants"),
            Tab::Preview => (
                "Preview",
                "Inspect the canonical JSON and validate the document",
            ),
        };
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .size(if theme::is_windows_2000() { 14.0 } else { 24.0 })
                        .strong()
                        .color(theme::text_color()),
                );
                ui.label(
                    egui::RichText::new(subtitle)
                        .size(11.5)
                        .color(theme::muted()),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.modified {
                    theme::badge(ui, "UNSAVED CHANGES", theme::gold());
                } else {
                    theme::badge(ui, "UP TO DATE", theme::success());
                }
            });
        });
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(12.0);

        match self.active_tab {
            Tab::Play => self.show_play_tab(ui),
            Tab::Preview => self.show_preview_tab(ui),
            tab => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match tab {
                        Tab::Info => self.show_info_tab(ui),
                        Tab::Rooms => self.show_rooms_tab(ui),
                        Tab::Items => self.show_items_tab(ui),
                        Tab::Monsters => self.show_monsters_tab(ui),
                        Tab::Quests => self.show_quests_tab(ui),
                        Tab::Shops => self.show_shops_tab(ui),
                        Tab::Play | Tab::Preview => {}
                    });
            }
        }
    }

    fn show_play_tab(&mut self, ui: &mut egui::Ui) {
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

    fn show_info_tab(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;
        ui.columns(2, |columns| {
            theme::card(
                &mut columns[0],
                "Identity",
                Some("What players and tooling see first"),
                |ui| {
                    egui::Grid::new("info_identity")
                        .num_columns(2)
                        .spacing([12.0, 9.0])
                        .show(ui, |ui| {
                            theme::field_label(ui, "Title");
                            changed |= ui
                                .add(
                                    egui::TextEdit::singleline(&mut self.adventure.title)
                                        .hint_text("The name of your adventure")
                                        .desired_width(320.0),
                                )
                                .changed();
                            ui.end_row();

                            theme::field_label(ui, "Identifier");
                            changed |= ui
                                .add(
                                    egui::TextEdit::singleline(&mut self.adventure.id)
                                        .hint_text("stable-id")
                                        .desired_width(320.0),
                                )
                                .changed();
                            ui.end_row();

                            theme::field_label(ui, "Author");
                            let mut author = self.adventure.author.clone().unwrap_or_default();
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut author)
                                        .hint_text("Your name")
                                        .desired_width(320.0),
                                )
                                .changed()
                            {
                                self.adventure.author = if author.trim().is_empty() {
                                    None
                                } else {
                                    Some(author)
                                };
                                changed = true;
                            }
                            ui.end_row();
                        });
                    theme::field_label(ui, "Description");
                    changed |= ui
                        .add(
                            egui::TextEdit::multiline(&mut self.adventure.description)
                                .hint_text("A short pitch for the adventure")
                                .desired_rows(4)
                                .desired_width(f32::INFINITY),
                        )
                        .changed();
                },
            );
            columns[0].add_space(10.0);
            theme::card(
                &mut columns[0],
                "Presentation",
                Some("The opening banner and introduction"),
                |ui| {
                    theme::field_label(ui, "Introduction");
                    changed |= ui
                        .add(
                            egui::TextEdit::multiline(&mut self.adventure.intro)
                                .hint_text("Set the scene before the first room description…")
                                .desired_rows(7)
                                .desired_width(f32::INFINITY),
                        )
                        .changed();
                },
            );

            theme::card(
                &mut columns[1],
                "Starting state",
                Some("Where the player begins and what they carry"),
                |ui| {
                    ui.horizontal(|ui| {
                        theme::field_label(ui, "Start room");
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.adventure.start_room)
                                    .range(0..=10_000),
                            )
                            .changed();
                        if ui.add(theme::ghost_button("Use first room")).clicked()
                            && let Some(room) = self.adventure.rooms.first()
                        {
                            self.adventure.start_room = room.id;
                            changed = true;
                        }
                    });
                    ui.add_space(6.0);
                    theme::field_label(ui, "Starting inventory");
                    if self.adventure.player_start_inventory.is_empty() {
                        ui.label(
                            egui::RichText::new(
                                "Empty. Set an item's location to 0 in the Items tab to grant it at the start.",
                            )
                            .size(11.5)
                            .color(theme::muted()),
                        );
                    } else {
                        for entry in &self.adventure.player_start_inventory {
                            let label = match entry {
                                InventoryRef::Id(id) => self
                                    .adventure
                                    .items
                                    .iter()
                                    .find(|item| item.id == *id)
                                    .map(|item| format!("[{}] {}", item.id, item.name))
                                    .unwrap_or_else(|| format!("item {}", id)),
                                InventoryRef::Item(item) => {
                                    format!("[{}] {}", item.id, item.name)
                                }
                            };
                            ui.label(egui::RichText::new(format!("• {}", label)).size(12.0));
                        }
                    }
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "{} rooms · {} items · {} monsters · {} quests · {} shops",
                            self.adventure.rooms.len(),
                            self.adventure.items.len(),
                            self.adventure.monsters.len(),
                            self.adventure.quests.len(),
                            self.adventure.shops.len()
                        ))
                        .size(11.0)
                        .color(theme::muted()),
                    );
                },
            );
            columns[1].add_space(10.0);
            theme::card(
                &mut columns[1],
                "Ending",
                Some("Optional victory conditions and closing text"),
                |ui| {
                    let mut has_ending = self.adventure.ending.is_some();
                    if ui.checkbox(&mut has_ending, "Enable victory conditions").changed() {
                        self.adventure.ending = if has_ending {
                            Some(EndingDef::default())
                        } else {
                            None
                        };
                        changed = true;
                    }
                    if let Some(ending) = &mut self.adventure.ending {
                        theme::field_label(ui, "Victory text");
                        changed |= ui
                            .add(
                                egui::TextEdit::multiline(&mut ending.victory_text)
                                    .desired_rows(2)
                                    .desired_width(f32::INFINITY),
                            )
                            .changed();
                        theme::field_label(ui, "Defeat text");
                        let mut defeat = ending.defeat_text.clone().unwrap_or_default();
                        if ui
                            .add(
                                egui::TextEdit::multiline(&mut defeat)
                                    .desired_rows(2)
                                    .desired_width(f32::INFINITY),
                            )
                            .changed()
                        {
                            ending.defeat_text = if defeat.trim().is_empty() {
                                None
                            } else {
                                Some(defeat)
                            };
                            changed = true;
                        }
                        changed |= ui
                            .checkbox(
                                &mut ending.conditions.all_quests,
                                "All quests must be completed",
                            )
                            .changed();

                        theme::field_label(ui, "Required room IDs (comma separated)");
                        let mut rooms_text = ending
                            .conditions
                            .rooms
                            .iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut rooms_text)
                                    .desired_width(f32::INFINITY),
                            )
                            .changed()
                        {
                            ending.conditions.rooms = rooms_text
                                .split(',')
                                .filter_map(|value| value.trim().parse::<i32>().ok())
                                .collect();
                            changed = true;
                        }

                        theme::field_label(ui, "Required quest IDs (comma separated)");
                        let mut quests_text = ending
                            .conditions
                            .quests
                            .iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut quests_text)
                                    .desired_width(f32::INFINITY),
                            )
                            .changed()
                        {
                            ending.conditions.quests = quests_text
                                .split(',')
                                .filter_map(|value| value.trim().parse::<i32>().ok())
                                .collect();
                            changed = true;
                        }

                        theme::field_label(ui, "Required items (names, comma separated)");
                        let mut items_text = ending.conditions.items.join(", ");
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut items_text)
                                    .desired_width(f32::INFINITY),
                            )
                            .changed()
                        {
                            ending.conditions.items = items_text
                                .split(',')
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .map(str::to_string)
                                .collect();
                            changed = true;
                        }
                    }
                },
            );
        });
        if changed {
            self.mark_modified();
        }
    }

    fn show_rooms_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Search")
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.room_filter)
                    .hint_text("Filter by name, description or ID")
                    .desired_width(280.0),
            );
            if ui.button("Clear").clicked() {
                self.room_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Room").clicked() {
                self.add_room();
            }
            if ui.button("Delete Room").clicked() && self.selected_room.is_some() {
                self.delete_room();
            }
        });

        ui.separator();

        ui.add_space(10.0);
        theme::card_panel(ui, |ui| {
            let content_height = 340.0;
            ui.columns(2, |columns| {
                columns[0].set_min_height(content_height);
                columns[1].set_min_height(content_height);
                // Room list
                theme::section_heading(&mut columns[0], "Rooms", Some("Select a room to edit"));
                let room_filter = self.room_filter.trim().to_lowercase();
                let visible_rooms = self
                    .adventure
                    .rooms
                    .iter()
                    .filter(|room| {
                        room_filter.is_empty()
                            || room.name.to_lowercase().contains(&room_filter)
                            || room.description.to_lowercase().contains(&room_filter)
                            || room.id.to_string().contains(&room_filter)
                    })
                    .count();
                columns[0].label(format!(
                    "Showing {} of {}",
                    visible_rooms,
                    self.adventure.rooms.len()
                ));
                egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                    for (i, room) in self.adventure.rooms.iter().enumerate() {
                        if !room_filter.is_empty()
                            && !room.name.to_lowercase().contains(&room_filter)
                            && !room.description.to_lowercase().contains(&room_filter)
                            && !room.id.to_string().contains(&room_filter)
                        {
                            continue;
                        }
                        let selected = self.selected_room == Some(i);
                        if ui
                            .add(
                                egui::Button::new(format!("{}: {}", room.id, room.name))
                                    .selected(selected),
                            )
                            .clicked()
                        {
                            self.selected_room = Some(i);
                        }
                    }
                });

                // Room editor
                theme::section_heading(&mut columns[1], "Room details", None);
                if let Some(room_idx) = self.selected_room {
                    if let Some(room) = self.adventure.rooms.get_mut(room_idx) {
                        let mut changed = false;
                        egui::Grid::new("room_grid")
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .show(&mut columns[1], |ui| {
                                ui.label("ID:");
                                changed |= ui.add(egui::DragValue::new(&mut room.id)).changed();
                                ui.end_row();

                                ui.label("Name:");
                                changed |= ui.text_edit_singleline(&mut room.name).changed();
                                ui.end_row();

                                ui.label("Dark:");
                                changed |= ui.checkbox(&mut room.is_dark, "").changed();
                                ui.end_row();
                            });
                        theme::field_label(&mut columns[1], "Description");
                        changed |= columns[1]
                            .add(
                                egui::TextEdit::multiline(&mut room.description)
                                    .desired_rows(3)
                                    .desired_width(f32::INFINITY),
                            )
                            .changed();

                        columns[1].separator();
                        columns[1].label("Exits:");
                        let exit_dirs: Vec<String> = room.exits.keys().cloned().collect();
                        let mut remove_dir: Option<String> = None;
                        for direction in &exit_dirs {
                            if let Some(exit) = room.exits.get_mut(direction) {
                                let mut is_locked = exit.lock().is_some_and(|lock| lock.locked);
                                let mut key_text = exit
                                    .lock()
                                    .and_then(|lock| lock.key.clone())
                                    .unwrap_or_default();
                                columns[1].horizontal(|ui| {
                                    ui.label(direction.as_str());
                                    ui.label("\u{2192}");
                                    if ui
                                        .add(egui::DragValue::new(exit.destination_mut()))
                                        .changed()
                                    {
                                        changed = true;
                                    }
                                    if ui.checkbox(&mut is_locked, "locked").changed() {
                                        exit.set_locked(
                                            is_locked,
                                            if is_locked {
                                                Some(key_text.clone())
                                            } else {
                                                None
                                            },
                                        );
                                        changed = true;
                                    }
                                    if is_locked && ui.text_edit_singleline(&mut key_text).changed()
                                    {
                                        exit.set_locked(true, Some(key_text.clone()));
                                        changed = true;
                                    }
                                    if ui.button("Remove").clicked() {
                                        remove_dir = Some(direction.clone());
                                    }
                                });
                            }
                        }
                        if let Some(dir) = remove_dir {
                            room.exits.remove(&dir);
                            changed = true;
                        }
                        columns[1].horizontal(|ui| {
                            egui::ComboBox::from_id_salt("add_exit_dir")
                                .selected_text(if self.new_exit_direction.is_empty() {
                                    "direction"
                                } else {
                                    &self.new_exit_direction
                                })
                                .show_ui(ui, |ui| {
                                    for d in &["north", "south", "east", "west", "up", "down"] {
                                        ui.selectable_value(
                                            &mut self.new_exit_direction,
                                            d.to_string(),
                                            *d,
                                        );
                                    }
                                });
                            ui.add(egui::DragValue::new(&mut self.new_exit_target).prefix("room "));
                            if ui.button("Add Exit").clicked()
                                && !self.new_exit_direction.is_empty()
                            {
                                room.exits.insert(
                                    self.new_exit_direction.clone(),
                                    ExitDef::Plain(self.new_exit_target),
                                );
                                changed = true;
                            }
                        });

                        columns[1].separator();
                        columns[1].label("Items in this room (IDs, comma separated):");
                        let mut items_text = room
                            .items
                            .iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        if columns[1].text_edit_singleline(&mut items_text).changed() {
                            room.items = items_text
                                .split(',')
                                .filter_map(|value| value.trim().parse::<i32>().ok())
                                .collect();
                            changed = true;
                        }

                        if changed {
                            self.mark_modified();
                        }
                    }
                } else {
                    columns[1].label("Select a room to edit");
                }
            });
        });
    }

    fn show_items_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Search")
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.item_filter)
                    .hint_text("Filter items")
                    .desired_width(280.0),
            );
            if ui.button("Clear").clicked() {
                self.item_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Item").clicked() {
                self.add_item();
            }
            if ui.button("Delete Item").clicked() && self.selected_item.is_some() {
                self.delete_item();
            }
        });

        ui.separator();

        ui.add_space(10.0);
        theme::card_panel(ui, |ui| {
            let content_height = 340.0;
            ui.columns(2, |columns| {
                columns[0].set_min_height(content_height);
                columns[1].set_min_height(content_height);
                // Item list
                theme::section_heading(&mut columns[0], "Items", Some("Select an item to edit"));
                let item_filter = self.item_filter.trim().to_lowercase();
                let visible_items = self
                    .adventure
                    .items
                    .iter()
                    .filter(|item| {
                        item_filter.is_empty()
                            || item.name.to_lowercase().contains(&item_filter)
                            || item.description.to_lowercase().contains(&item_filter)
                            || item.id.to_string().contains(&item_filter)
                    })
                    .count();
                columns[0].label(format!(
                    "Showing {} of {}",
                    visible_items,
                    self.adventure.items.len()
                ));
                egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                    for (i, item) in self.adventure.items.iter().enumerate() {
                        if !item_filter.is_empty()
                            && !item.name.to_lowercase().contains(&item_filter)
                            && !item.description.to_lowercase().contains(&item_filter)
                            && !item.id.to_string().contains(&item_filter)
                        {
                            continue;
                        }
                        let selected = self.selected_item == Some(i);
                        if ui
                            .add(
                                egui::Button::new(format!("{}: {}", item.id, item.name))
                                    .selected(selected),
                            )
                            .clicked()
                        {
                            self.selected_item = Some(i);
                        }
                    }
                });

                // Item editor
                theme::section_heading(&mut columns[1], "Item details", None);
                if let Some(item_idx) = self.selected_item {
                    if let Some(item) = self.adventure.items.get_mut(item_idx) {
                        let mut changed = false;
                        egui::Grid::new("item_grid")
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .show(&mut columns[1], |ui| {
                                ui.label("ID:");
                                changed |= ui.add(egui::DragValue::new(&mut item.id)).changed();
                                ui.end_row();

                                ui.label("Name:");
                                changed |= ui.text_edit_singleline(&mut item.name).changed();
                                ui.end_row();

                                ui.label("Type:");
                                egui::ComboBox::from_id_salt("item_type")
                                    .selected_text(format!("{:?}", item.item_type))
                                    .show_ui(ui, |ui: &mut egui::Ui| {
                                        for variant in [
                                            ItemType::Normal,
                                            ItemType::Weapon,
                                            ItemType::Armor,
                                            ItemType::Treasure,
                                            ItemType::Readable,
                                            ItemType::Edible,
                                            ItemType::Drinkable,
                                            ItemType::Container,
                                        ] {
                                            changed |= ui
                                                .selectable_value(
                                                    &mut item.item_type,
                                                    variant.clone(),
                                                    format!("{variant:?}"),
                                                )
                                                .changed();
                                        }
                                    });
                                ui.end_row();

                                ui.label("Value:");
                                changed |= ui.add(egui::DragValue::new(&mut item.value)).changed();
                                ui.end_row();

                                ui.label("Weight:");
                                changed |= ui.add(egui::DragValue::new(&mut item.weight)).changed();
                                ui.end_row();

                                ui.label("Location:");
                                changed |=
                                    ui.add(egui::DragValue::new(&mut item.location)).changed();
                                ui.end_row();

                                ui.label("");
                                ui.label(
                                    "0 = carried, -1 = worn, -2 = shop, otherwise room/monster ID",
                                );
                                ui.end_row();

                                ui.label("Takeable:");
                                changed |= ui.checkbox(&mut item.is_takeable, "").changed();
                                ui.end_row();

                                ui.label("Light source:");
                                changed |= ui.checkbox(&mut item.is_light_source, "").changed();
                                ui.end_row();

                                if item.item_type == ItemType::Container {
                                    ui.label("Container capacity:");
                                    changed |= ui
                                        .add(egui::DragValue::new(&mut item.container_capacity))
                                        .changed();
                                    ui.end_row();
                                }

                                ui.label("Is Weapon:");
                                changed |= ui.checkbox(&mut item.is_weapon, "").changed();
                                ui.end_row();

                                if item.is_weapon {
                                    ui.label("Weapon Type (1-5):");
                                    changed |= ui
                                        .add(
                                            egui::DragValue::new(&mut item.weapon_type)
                                                .range(1..=5),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label("Damage Dice:");
                                    changed |= ui
                                        .add(
                                            egui::DragValue::new(&mut item.weapon_dice)
                                                .range(1..=10),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label("Damage Sides:");
                                    changed |= ui
                                        .add(
                                            egui::DragValue::new(&mut item.weapon_sides)
                                                .range(2..=20),
                                        )
                                        .changed();
                                    ui.end_row();
                                }

                                ui.label("Is Armor:");
                                changed |= ui.checkbox(&mut item.is_armor, "").changed();
                                ui.end_row();

                                if item.is_armor {
                                    ui.label("Armor Value:");
                                    changed |= ui
                                        .add(
                                            egui::DragValue::new(&mut item.armor_value)
                                                .range(0..=20),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label("Wearable:");
                                    changed |= ui.checkbox(&mut item.is_wearable, "").changed();
                                    ui.end_row();
                                }
                            });
                        theme::field_label(&mut columns[1], "Description");
                        changed |= columns[1]
                            .add(
                                egui::TextEdit::multiline(&mut item.description)
                                    .desired_rows(3)
                                    .desired_width(f32::INFINITY),
                            )
                            .changed();
                        if changed {
                            self.mark_modified();
                        }
                    }
                } else {
                    columns[1].label("Select an item to edit");
                }
            });
        });
    }

    fn show_monsters_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Search")
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.monster_filter)
                    .hint_text("Filter monsters")
                    .desired_width(280.0),
            );
            if ui.button("Clear").clicked() {
                self.monster_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Monster").clicked() {
                self.add_monster();
            }
            if ui.button("Delete Monster").clicked() && self.selected_monster.is_some() {
                self.delete_monster();
            }
        });

        ui.separator();

        ui.add_space(10.0);
        theme::card_panel(ui, |ui| {
            let content_height = 340.0;
            ui.columns(2, |columns| {
                columns[0].set_min_height(content_height);
                columns[1].set_min_height(content_height);
                // Monster list
                theme::section_heading(
                    &mut columns[0],
                    "Monsters",
                    Some("Select a monster to edit"),
                );
                let monster_filter = self.monster_filter.trim().to_lowercase();
                let visible_monsters = self
                    .adventure
                    .monsters
                    .iter()
                    .filter(|monster| {
                        monster_filter.is_empty()
                            || monster.name.to_lowercase().contains(&monster_filter)
                            || monster.description.to_lowercase().contains(&monster_filter)
                            || monster.id.to_string().contains(&monster_filter)
                    })
                    .count();
                columns[0].label(format!(
                    "Showing {} of {}",
                    visible_monsters,
                    self.adventure.monsters.len()
                ));
                egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                    for (i, monster) in self.adventure.monsters.iter().enumerate() {
                        if !monster_filter.is_empty()
                            && !monster.name.to_lowercase().contains(&monster_filter)
                            && !monster.description.to_lowercase().contains(&monster_filter)
                            && !monster.id.to_string().contains(&monster_filter)
                        {
                            continue;
                        }
                        let selected = self.selected_monster == Some(i);
                        if ui
                            .add(
                                egui::Button::new(format!("{}: {}", monster.id, monster.name))
                                    .selected(selected),
                            )
                            .clicked()
                        {
                            self.selected_monster = Some(i);
                        }
                    }
                });

                // Monster editor
                theme::section_heading(&mut columns[1], "Monster details", None);
                if let Some(monster_idx) = self.selected_monster {
                    if let Some(monster) = self.adventure.monsters.get_mut(monster_idx) {
                        let mut changed = false;
                        egui::Grid::new("monster_grid")
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .show(&mut columns[1], |ui| {
                                ui.label("ID:");
                                changed |= ui.add(egui::DragValue::new(&mut monster.id)).changed();
                                ui.end_row();

                                ui.label("Name:");
                                changed |= ui.text_edit_singleline(&mut monster.name).changed();
                                ui.end_row();

                                ui.label("Hardiness:");
                                changed |= ui
                                    .add(egui::DragValue::new(&mut monster.hardiness))
                                    .changed();
                                ui.end_row();

                                ui.label("Agility:");
                                changed |=
                                    ui.add(egui::DragValue::new(&mut monster.agility)).changed();
                                ui.end_row();

                                ui.label("Gold:");
                                changed |=
                                    ui.add(egui::DragValue::new(&mut monster.gold)).changed();
                                ui.end_row();

                                ui.label("Room ID:");
                                changed |=
                                    ui.add(egui::DragValue::new(&mut monster.room_id)).changed();
                                ui.end_row();

                                ui.label("Friendliness:");
                                egui::ComboBox::from_id_salt("monster_status")
                                    .selected_text(format!("{:?}", monster.friendliness))
                                    .show_ui(ui, |ui: &mut egui::Ui| {
                                        changed |= ui
                                            .selectable_value(
                                                &mut monster.friendliness,
                                                MonsterStatus::Neutral,
                                                "Neutral",
                                            )
                                            .changed();
                                        changed |= ui
                                            .selectable_value(
                                                &mut monster.friendliness,
                                                MonsterStatus::Friendly,
                                                "Friendly",
                                            )
                                            .changed();
                                        changed |= ui
                                            .selectable_value(
                                                &mut monster.friendliness,
                                                MonsterStatus::Hostile,
                                                "Hostile",
                                            )
                                            .changed();
                                    });
                                ui.end_row();

                                ui.label("Courage:");
                                changed |=
                                    ui.add(egui::DragValue::new(&mut monster.courage)).changed();
                                ui.end_row();

                                ui.label("Armor worn:");
                                changed |= ui
                                    .add(egui::DragValue::new(&mut monster.armor_worn))
                                    .changed();
                                ui.end_row();

                                ui.label("Weapon ID:");
                                let mut has_weapon = monster.weapon_id.is_some();
                                ui.horizontal(|ui| {
                                    if ui.checkbox(&mut has_weapon, "Has weapon").changed() {
                                        monster.weapon_id = if has_weapon { Some(1) } else { None };
                                        changed = true;
                                    }
                                    if let Some(weapon_id) = &mut monster.weapon_id {
                                        changed |=
                                            ui.add(egui::DragValue::new(weapon_id)).changed();
                                    }
                                });
                                ui.end_row();

                                ui.label("Loot (item IDs):");
                                let mut loot_text = monster
                                    .loot
                                    .iter()
                                    .map(|id| id.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                if ui.text_edit_singleline(&mut loot_text).changed() {
                                    monster.loot = loot_text
                                        .split(',')
                                        .filter_map(|value| value.trim().parse::<i32>().ok())
                                        .collect();
                                    changed = true;
                                }
                                ui.end_row();

                                ui.label("Dialogue:");
                                let mut has_dialogue = monster.dialogue.is_some();
                                if ui.checkbox(&mut has_dialogue, "Has dialogue").changed() {
                                    monster.dialogue = if has_dialogue {
                                        Some(DialogueDef::default())
                                    } else {
                                        None
                                    };
                                    changed = true;
                                }
                                ui.end_row();

                                if let Some(dialogue) = &mut monster.dialogue {
                                    ui.label("Greeting:");
                                    changed |=
                                        ui.text_edit_multiline(&mut dialogue.greeting).changed();
                                    ui.end_row();

                                    ui.label("Unknown topic reply:");
                                    let mut unknown = dialogue.unknown.clone().unwrap_or_default();
                                    if ui.text_edit_multiline(&mut unknown).changed() {
                                        dialogue.unknown = if unknown.trim().is_empty() {
                                            None
                                        } else {
                                            Some(unknown)
                                        };
                                        changed = true;
                                    }
                                    ui.end_row();

                                    ui.label("Topics (one per line, 'topic: reply'):");
                                    let mut topics_text = dialogue
                                        .topics
                                        .iter()
                                        .map(|(topic, reply)| format!("{}: {}", topic, reply))
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    if ui.text_edit_multiline(&mut topics_text).changed() {
                                        dialogue.topics.clear();
                                        for line in topics_text.lines() {
                                            if let Some((topic, reply)) = line.split_once(':') {
                                                let topic = topic.trim();
                                                if !topic.is_empty() {
                                                    dialogue.topics.insert(
                                                        topic.to_string(),
                                                        reply.trim().to_string(),
                                                    );
                                                }
                                            }
                                        }
                                        changed = true;
                                    }
                                    ui.end_row();
                                }
                            });
                        theme::field_label(&mut columns[1], "Description");
                        changed |= columns[1]
                            .add(
                                egui::TextEdit::multiline(&mut monster.description)
                                    .desired_rows(3)
                                    .desired_width(f32::INFINITY),
                            )
                            .changed();
                        if changed {
                            self.mark_modified();
                        }
                    }
                } else {
                    columns[1].label("Select a monster to edit");
                }
            });
        });
    }

    fn show_quests_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Search")
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.quest_filter)
                    .hint_text("Filter quests")
                    .desired_width(280.0),
            );
            if ui.button("Clear").clicked() {
                self.quest_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Quest").clicked() {
                self.add_quest();
            }
            if ui.button("Delete Quest").clicked() && self.selected_quest.is_some() {
                self.delete_quest();
            }
        });

        ui.separator();

        ui.add_space(10.0);
        theme::card_panel(ui, |ui| {
            let content_height = 340.0;
            ui.columns(2, |columns| {
                columns[0].set_min_height(content_height);
                columns[1].set_min_height(content_height);
                // Quest list
                theme::section_heading(&mut columns[0], "Quests", Some("Select a quest to edit"));
                let quest_filter = self.quest_filter.trim().to_lowercase();
                let visible_quests = self
                    .adventure
                    .quests
                    .iter()
                    .filter(|quest| {
                        quest_filter.is_empty()
                            || quest.title.to_lowercase().contains(&quest_filter)
                            || quest.description.to_lowercase().contains(&quest_filter)
                            || quest.id.to_string().contains(&quest_filter)
                    })
                    .count();
                columns[0].label(format!(
                    "Showing {} of {}",
                    visible_quests,
                    self.adventure.quests.len()
                ));
                egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                    for (i, quest) in self.adventure.quests.iter().enumerate() {
                        if !quest_filter.is_empty()
                            && !quest.title.to_lowercase().contains(&quest_filter)
                            && !quest.description.to_lowercase().contains(&quest_filter)
                            && !quest.id.to_string().contains(&quest_filter)
                        {
                            continue;
                        }
                        let selected = self.selected_quest == Some(i);
                        if ui
                            .add(
                                egui::Button::new(format!("{}: {}", quest.id, quest.title))
                                    .selected(selected),
                            )
                            .clicked()
                        {
                            self.selected_quest = Some(i);
                        }
                    }
                });

                // Quest editor
                theme::section_heading(&mut columns[1], "Quest details", None);
                if let Some(quest_idx) = self.selected_quest {
                    if let Some(quest) = self.adventure.quests.get_mut(quest_idx) {
                        let mut changed = false;
                        columns[1].horizontal(|ui| {
                            ui.label("ID:");
                            changed |= ui.add(egui::DragValue::new(&mut quest.id)).changed();
                            ui.label("Title:");
                            changed |= ui.text_edit_singleline(&mut quest.title).changed();
                        });
                        columns[1].horizontal(|ui| {
                            ui.label("Giver NPC:");
                            changed |= ui.text_edit_singleline(&mut quest.giver_npc).changed();
                            ui.label("Difficulty:");
                            egui::ComboBox::from_id_salt("quest_difficulty")
                                .selected_text(format!("{:?}", quest.difficulty))
                                .show_ui(ui, |ui| {
                                    changed |= ui
                                        .selectable_value(
                                            &mut quest.difficulty,
                                            loresmith_rs::QuestDifficultyDef::Easy,
                                            "Easy",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut quest.difficulty,
                                            loresmith_rs::QuestDifficultyDef::Moderate,
                                            "Moderate",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut quest.difficulty,
                                            loresmith_rs::QuestDifficultyDef::Challenging,
                                            "Challenging",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut quest.difficulty,
                                            loresmith_rs::QuestDifficultyDef::Hard,
                                            "Hard",
                                        )
                                        .changed();
                                });
                        });
                        columns[1].label("Description:");
                        changed |= columns[1]
                            .text_edit_multiline(&mut quest.description)
                            .changed();

                        if quest.stages.is_empty() {
                            quest.stages.push(StageDef {
                                stage_id: "main".to_string(),
                                stage_number: 1,
                                title: "Main Objectives".to_string(),
                                description: quest.description.clone(),
                                objectives: quest.objectives.clone(),
                                stage_reward_xp: quest.rewards.xp,
                                extra: Default::default(),
                            });
                        }

                        let mut remove_stage: Option<usize> = None;
                        for (stage_idx, stage) in quest.stages.iter_mut().enumerate() {
                            columns[1].group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Stage {}", stage.stage_number));
                                    changed |= ui.text_edit_singleline(&mut stage.title).changed();
                                    ui.label("Stage XP:");
                                    changed |= ui
                                        .add(egui::DragValue::new(&mut stage.stage_reward_xp))
                                        .changed();
                                    if ui.button("Remove stage").clicked() {
                                        remove_stage = Some(stage_idx);
                                    }
                                });
                                ui.label("Objectives:");
                                let mut remove_objective: Option<usize> = None;
                                for (objective_idx, objective) in
                                    stage.objectives.iter_mut().enumerate()
                                {
                                    let objective =
                                        ensure_structured_objective(objective, objective_idx);
                                    ui.horizontal(|ui| {
                                        egui::ComboBox::from_id_salt(format!(
                                            "objective_kind_{}_{}",
                                            stage_idx, objective_idx
                                        ))
                                        .selected_text(objective_kind_label(objective.kind))
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                for kind in [
                                                    ObjectiveKind::KillMonster,
                                                    ObjectiveKind::CollectItem,
                                                    ObjectiveKind::ReachRoom,
                                                    ObjectiveKind::TalkToNpc,
                                                    ObjectiveKind::Discover,
                                                ] {
                                                    changed |= ui
                                                        .selectable_value(
                                                            &mut objective.kind,
                                                            kind,
                                                            objective_kind_label(kind),
                                                        )
                                                        .changed();
                                                }
                                            },
                                        );
                                        let mut target = objective
                                            .target
                                            .clone()
                                            .unwrap_or_else(|| objective.target_text());
                                        let target_width =
                                            (ui.available_width() - 210.0).max(120.0);
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut target)
                                                    .hint_text("target")
                                                    .desired_width(target_width),
                                            )
                                            .changed()
                                        {
                                            objective.target_id = Some(
                                                target
                                                    .parse::<i64>()
                                                    .map(|number| {
                                                        serde_json::Value::Number(number.into())
                                                    })
                                                    .unwrap_or_else(|_| {
                                                        serde_json::Value::String(target.clone())
                                                    }),
                                            );
                                            objective.target = Some(target);
                                            changed = true;
                                        }
                                        changed |= ui
                                            .add(
                                                egui::DragValue::new(&mut objective.count)
                                                    .range(1..=999),
                                            )
                                            .changed();
                                        if ui.button("Remove").clicked() {
                                            remove_objective = Some(objective_idx);
                                        }
                                    });
                                    changed |= ui
                                        .add(
                                            egui::TextEdit::singleline(&mut objective.description)
                                                .hint_text("Objective description")
                                                .desired_width(f32::INFINITY),
                                        )
                                        .changed();
                                    ui.add_space(4.0);
                                }
                                if let Some(index) = remove_objective {
                                    stage.objectives.remove(index);
                                    changed = true;
                                }
                                if ui.button("Add Objective").clicked() {
                                    stage.objectives.push(ObjectiveDef::Structured(
                                        StructuredObjective {
                                            kind: ObjectiveKind::KillMonster,
                                            description: "Defeat an enemy".to_string(),
                                            target: Some("enemy".to_string()),
                                            target_id: Some(serde_json::Value::String(
                                                "enemy".to_string(),
                                            )),
                                            count: 1,
                                            ..StructuredObjective::default()
                                        },
                                    ));
                                    changed = true;
                                }
                            });
                        }
                        if let Some(index) = remove_stage
                            && quest.stages.len() > 1
                        {
                            quest.stages.remove(index);
                            changed = true;
                        }
                        if columns[1].button("Add Stage").clicked() {
                            let number = quest.stages.len() as i32 + 1;
                            quest.stages.push(StageDef {
                                stage_id: format!("stage_{}", number),
                                stage_number: number,
                                title: format!("Stage {}", number),
                                ..StageDef::default()
                            });
                            changed = true;
                        }

                        // Mirror the first stage into the legacy objectives field.
                        if let Some(first) = quest.stages.first() {
                            quest.objectives = first.objectives.clone();
                        }

                        columns[1].separator();
                        columns[1].horizontal(|ui| {
                            ui.label("Gold reward:");
                            changed |= ui
                                .add(egui::DragValue::new(&mut quest.rewards.gold))
                                .changed();
                            ui.label("XP reward:");
                            changed |= ui
                                .add(egui::DragValue::new(&mut quest.rewards.xp))
                                .changed();
                        });
                        columns[1].label("Reward items (IDs or names, comma separated):");
                        let mut reward_items = quest
                            .rewards
                            .items
                            .iter()
                            .map(|item| match item {
                                RewardItem::Id(id) => id.to_string(),
                                RewardItem::Name(name) => name.clone(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        if columns[1].text_edit_singleline(&mut reward_items).changed() {
                            quest.rewards.items = reward_items
                                .split(',')
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .map(|value| {
                                    value
                                        .parse::<i32>()
                                        .map(RewardItem::Id)
                                        .unwrap_or_else(|_| RewardItem::Name(value.to_string()))
                                })
                                .collect();
                            changed = true;
                        }
                        columns[1].label("Reputation rewards (Faction: amount, comma separated):");
                        let mut reputation_text = quest
                            .rewards
                            .reputation
                            .iter()
                            .map(|(faction, amount)| format!("{}: {}", faction, amount))
                            .collect::<Vec<_>>()
                            .join(", ");
                        if columns[1]
                            .text_edit_singleline(&mut reputation_text)
                            .changed()
                        {
                            quest.rewards.reputation.clear();
                            for entry in reputation_text.split(',') {
                                if let Some((faction, amount)) = entry.split_once(':')
                                    && let Ok(amount) = amount.trim().parse::<i32>()
                                {
                                    quest
                                        .rewards
                                        .reputation
                                        .insert(faction.trim().to_string(), amount);
                                }
                            }
                            changed = true;
                        }
                        quest.rewards_gold = Some(quest.rewards.gold);
                        quest.rewards_xp = Some(quest.rewards.xp);
                        if changed {
                            self.mark_modified();
                        }
                    }
                } else {
                    columns[1].label("Select a quest to edit");
                }
            });
        });
    }

    fn show_shops_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Search")
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.shop_filter)
                    .hint_text("Filter shops")
                    .desired_width(280.0),
            );
            if ui.button("Clear").clicked() {
                self.shop_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Shop").clicked() {
                self.add_shop();
            }
            if ui.button("Delete Shop").clicked() && self.selected_shop.is_some() {
                self.delete_shop();
            }
        });

        ui.separator();
        let item_names: Vec<(i32, String)> = self
            .adventure
            .items
            .iter()
            .map(|item| (item.id, item.name.clone()))
            .collect();

        ui.add_space(10.0);
        theme::card_panel(ui, |ui| {
            let content_height = 340.0;
            ui.columns(2, |columns| {
                columns[0].set_min_height(content_height);
                columns[1].set_min_height(content_height);
                theme::section_heading(&mut columns[0], "Shops", Some("Select a shop to edit"));
                let filter = self.shop_filter.trim().to_lowercase();
                egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                    for (index, shop) in self.adventure.shops.iter().enumerate() {
                        if !filter.is_empty()
                            && !shop.name.to_lowercase().contains(&filter)
                            && !shop.id.to_string().contains(&filter)
                        {
                            continue;
                        }
                        if ui
                            .add(
                                egui::Button::new(format!("{}: {}", shop.id, shop.name))
                                    .selected(self.selected_shop == Some(index)),
                            )
                            .clicked()
                        {
                            self.selected_shop = Some(index);
                        }
                    }
                });

                theme::section_heading(&mut columns[1], "Shop details", None);
                let Some(index) = self.selected_shop else {
                    columns[1].label("Select a shop to edit");
                    return;
                };
                let Some(shop) = self.adventure.shops.get_mut(index) else {
                    columns[1].label("Select a shop to edit");
                    return;
                };
                let mut changed = false;
                egui::Grid::new("shop_grid")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(&mut columns[1], |ui| {
                        ui.label("ID:");
                        changed |= ui.add(egui::DragValue::new(&mut shop.id)).changed();
                        ui.end_row();
                        ui.label("Name:");
                        changed |= ui.text_edit_singleline(&mut shop.name).changed();
                        ui.end_row();
                        ui.label("Room ID:");
                        changed |= ui.add(egui::DragValue::new(&mut shop.room_id)).changed();
                        ui.end_row();
                        ui.label("Gold:");
                        changed |= ui.add(egui::DragValue::new(&mut shop.gold)).changed();
                        ui.end_row();
                        ui.label("Buy multiplier:");
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut shop.buy_multiplier)
                                    .speed(0.05)
                                    .range(0.01..=10.0),
                            )
                            .changed();
                        ui.end_row();
                        ui.label("Sell multiplier:");
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut shop.sell_multiplier)
                                    .speed(0.05)
                                    .range(0.0..=10.0),
                            )
                            .changed();
                        ui.end_row();
                    });
                columns[1].label(
                    egui::RichText::new(
                        "The player pays value × buy multiplier (reduced by charisma) and receives value × sell multiplier (increased by charisma). A shop never pays more than its own asking price for stocked goods.",
                    )
                    .size(11.0)
                    .color(theme::muted()),
                );

                columns[1].separator();
                columns[1].label("Stock:");
                let mut remove: Option<usize> = None;
                for (stock_index, stock) in shop.stock.iter_mut().enumerate() {
                    columns[1].horizontal(|ui| {
                        let label = item_names
                            .iter()
                            .find(|(id, _)| *id == stock.item_id)
                            .map(|(_, name)| name.clone())
                            .unwrap_or_else(|| format!("item {}", stock.item_id));
                        egui::ComboBox::from_id_salt(("shop_stock_item", stock_index))
                            .selected_text(label)
                            .show_ui(ui, |ui| {
                                for (id, name) in &item_names {
                                    changed |= ui
                                        .selectable_value(
                                            &mut stock.item_id,
                                            *id,
                                            name.as_str(),
                                        )
                                        .changed();
                                }
                            });
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut stock.quantity)
                                    .prefix("qty ")
                                    .range(0..=99),
                            )
                            .changed();
                        if ui.button("Remove").clicked() {
                            remove = Some(stock_index);
                        }
                    });
                }
                if let Some(index) = remove {
                    shop.stock.remove(index);
                    changed = true;
                }
                if columns[1].button("Add Stock").clicked() {
                    let item_id = item_names.first().map(|(id, _)| *id).unwrap_or(1);
                    shop.stock.push(ShopStockDef {
                        item_id,
                        quantity: 1,
                        ..ShopStockDef::default()
                    });
                    changed = true;
                }
                if changed {
                    self.mark_modified();
                }
            });
        });
    }

    fn show_preview_tab(&mut self, ui: &mut egui::Ui) {
        let report = self.adventure.report();
        ui.horizontal(|ui| {
            if ui.add(theme::primary_button("Copy JSON")).clicked() {
                self.copy_json_to_clipboard(ui.ctx());
            }
            if ui.add(theme::ghost_button("Show diff")).clicked() {
                self.show_json_diff();
            }
            if ui.add(theme::ghost_button("Save as JSON")).clicked() {
                self.save_adventure_as();
            }
            if ui.add(theme::ghost_button("Validate & Play")).clicked() && self.validate_adventure()
            {
                self.start_game();
                self.active_tab = Tab::Play;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (label, color) = if report.is_valid() {
                    ("VALID", theme::success())
                } else {
                    ("INVALID", theme::danger())
                };
                theme::badge(ui, label, color);
            });
        });
        ui.add_space(10.0);

        let content_height = ui.available_height().max(360.0);
        ui.columns(2, |columns| {
            columns[0].set_min_height(content_height);
            columns[1].set_min_height(content_height);
            theme::card(
                &mut columns[0],
                "Canonical JSON",
                Some("schema_version 2 · normalized on save"),
                |ui| {
                    theme::field_panel(ui, |ui| {
                            egui::ScrollArea::both()
                                .auto_shrink([false, false])
                                .max_height(content_height - 86.0)
                                .show(ui, |ui| {
                                    let json = self.generate_json_preview();
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(json)
                                                .monospace()
                                                .size(12.0)
                                                .color(theme::text_color()),
                                        )
                                        .wrap_mode(egui::TextWrapMode::Extend)
                                        .selectable(true),
                                    );
                                });
                        });
                },
            );
            theme::card(
                &mut columns[1],
                "Document health",
                Some("Validation and export"),
                |ui| {
                    let stats = [
                        ("ROOMS", self.adventure.rooms.len(), theme::accent()),
                        ("ITEMS", self.adventure.items.len(), theme::text_color()),
                        ("MONSTERS", self.adventure.monsters.len(), theme::gold()),
                        ("QUESTS", self.adventure.quests.len(), theme::success()),
                        ("SHOPS", self.adventure.shops.len(), theme::muted()),
                    ];
                    egui::Grid::new("preview_stats")
                        .num_columns(3)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            for (index, (label, value, color)) in stats.iter().enumerate() {
                                theme::stat_card(ui, label, &value.to_string(), *color);
                                if (index + 1).is_multiple_of(3) {
                                    ui.end_row();
                                }
                            }
                            if !stats.len().is_multiple_of(3) {
                                ui.end_row();
                            }
                        });
                    ui.add_space(10.0);
                    theme::field_label(ui, "VALIDATION");
                    if report.is_valid() && report.warnings.is_empty() {
                        ui.label(
                            egui::RichText::new("No errors or warnings.")
                                .size(12.0)
                                .color(theme::success()),
                        );
                    } else {
                        for error in &report.errors {
                            ui.label(
                                egui::RichText::new(format!("Error: {}", error))
                                    .size(12.0)
                                    .color(theme::danger()),
                            );
                        }
                        for warning in &report.warnings {
                            ui.label(
                                egui::RichText::new(format!("Warning: {}", warning))
                                    .size(12.0)
                                    .color(theme::gold()),
                            );
                        }
                    }
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);
                    theme::field_label(ui, "ACTIONS");
                    ui.horizontal(|ui| {
                        if ui.add(theme::ghost_button("Validate now")).clicked() {
                            self.validate_adventure();
                        }
                        if ui.add(theme::ghost_button("Copy JSON")).clicked() {
                            self.copy_json_to_clipboard(ui.ctx());
                        }
                    });
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(
                            "Saving normalizes the document, preserves unknown editor fields, and writes atomically.",
                        )
                        .size(11.0)
                        .color(theme::muted()),
                    );
                },
            );
        });
    }

    // File operations
    fn new_adventure(&mut self) {
        if self.modified {
            self.status =
                "Unsaved changes: save or close the current adventure before creating a new one."
                    .to_string();
            return;
        }
        self.adventure = starter_document();
        self.current_file = None;
        self.modified = false;
        self.last_saved_json = self.adventure.to_json_string().ok();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.frame_snapshot = None;
        self.selected_room = Some(0);
        self.selected_item = Some(0);
        self.selected_monster = Some(0);
        self.selected_quest = Some(0);
        self.selected_shop = None;
        self.status = "New adventure created".to_string();
    }

    fn open_adventure(&mut self) {
        if self.modified {
            self.status = "Unsaved changes: save before opening another adventure.".to_string();
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON files", &["json"][..])
            .add_filter("All files", &["*"][..])
            .pick_file()
        {
            match self.load_from_file(&path) {
                Ok(_) => {
                    self.current_file = Some(path.clone());
                    self.modified = false;
                    self.status = format!("Opened adventure: {}", path.display());
                }
                Err(e) => {
                    self.status = format!("Error opening file: {}", e);
                }
            }
        }
    }

    fn save_adventure(&mut self) -> bool {
        if let Some(path) = self.current_file.clone() {
            match self.save_to_file(&path) {
                Ok(_) => {
                    self.modified = false;
                    self.status = format!("Saved adventure: {}", path.display());
                    true
                }
                Err(e) => {
                    self.status = format!("Error saving file: {}", e);
                    false
                }
            }
        } else {
            self.save_adventure_as()
        }
    }

    fn save_adventure_as(&mut self) -> bool {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON files", &["json"][..])
            .add_filter("All files", &["*"][..])
            .save_file()
        {
            match self.save_to_file(&path) {
                Ok(_) => {
                    self.current_file = Some(path.clone());
                    self.modified = false;
                    self.status = format!("Saved adventure as: {}", path.display());
                    true
                }
                Err(e) => {
                    self.status = format!("Error saving file: {}", e);
                    false
                }
            }
        } else {
            self.status = "Save cancelled".to_string();
            false
        }
    }

    fn save_to_file(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.adventure.save_json_file(path)?;
        self.last_saved_json = self.adventure.to_json_string().ok();
        self.frame_snapshot = None;
        Ok(())
    }

    fn load_from_file(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let document = AdventureDocument::load_json_file(path)?;
        self.adventure = document;
        self.last_saved_json = self.adventure.to_json_string().ok();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.frame_snapshot = None;
        self.selected_room = if self.adventure.rooms.is_empty() {
            None
        } else {
            Some(0)
        };
        self.selected_item = if self.adventure.items.is_empty() {
            None
        } else {
            Some(0)
        };
        self.selected_monster = if self.adventure.monsters.is_empty() {
            None
        } else {
            Some(0)
        };
        self.selected_quest = if self.adventure.quests.is_empty() {
            None
        } else {
            Some(0)
        };
        self.selected_shop = if self.adventure.shops.is_empty() {
            None
        } else {
            Some(0)
        };
        Ok(())
    }

    fn validate_adventure(&mut self) -> bool {
        let report = self.adventure.report();
        if report.is_valid() {
            self.status = format!(
                "Valid: {} rooms, {} items, {} monsters, {} quests, {} shops{}",
                self.adventure.rooms.len(),
                self.adventure.items.len(),
                self.adventure.monsters.len(),
                self.adventure.quests.len(),
                self.adventure.shops.len(),
                if report.warnings.is_empty() {
                    String::new()
                } else {
                    format!(
                        " — {} warning(s): {}",
                        report.warnings.len(),
                        report.warnings.join("; ")
                    )
                }
            );
            true
        } else {
            self.status = format!(
                "Validation errors: {}{}",
                report.errors.join("; "),
                if report.warnings.is_empty() {
                    String::new()
                } else {
                    format!(" — warnings: {}", report.warnings.join("; "))
                }
            );
            false
        }
    }

    fn export_to_json(&mut self) {
        self.save_adventure_as();
    }

    fn show_about(&mut self) {
        self.status = "Loresmith Studio — complete adventure editor".to_string();
    }

    fn mark_modified(&mut self) {
        if !self.modified {
            let snapshot = self
                .frame_snapshot
                .take()
                .unwrap_or_else(|| self.adventure.clone());
            self.undo_stack.push(snapshot);
            if self.undo_stack.len() > 100 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }
        self.modified = true;
    }

    fn undo(&mut self) {
        if let Some(previous) = self.undo_stack.pop() {
            self.redo_stack.push(self.adventure.clone());
            self.adventure = previous;
            self.modified = true;
            self.frame_snapshot = None;
            self.status = "Undo".to_string();
        } else {
            self.status = "Nothing to undo".to_string();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.adventure.clone());
            self.adventure = next;
            self.modified = true;
            self.frame_snapshot = None;
            self.status = "Redo".to_string();
        } else {
            self.status = "Nothing to redo".to_string();
        }
    }

    // CRUD operations
    fn add_room(&mut self) {
        self.mark_modified();
        let id = self
            .adventure
            .rooms
            .iter()
            .map(|room| room.id)
            .max()
            .unwrap_or(0)
            + 1;
        self.adventure.rooms.push(RoomDef {
            id,
            name: format!("Room {}", id),
            description: "A new room".to_string(),
            ..RoomDef::default()
        });
        self.status = format!("Room {} added", id);
    }

    fn delete_room(&mut self) {
        let Some(idx) = self.selected_room else {
            return;
        };
        if self.adventure.rooms.len() <= 1 {
            self.status = "An adventure needs at least one room".to_string();
            return;
        }
        self.mark_modified();
        let removed = self.adventure.rooms.remove(idx);
        for room in &mut self.adventure.rooms {
            room.exits
                .retain(|_, exit| exit.destination() != removed.id);
        }
        for item in &mut self.adventure.items {
            if item.location == removed.id {
                item.location = 0;
            }
        }
        for monster in &mut self.adventure.monsters {
            if monster.room_id == removed.id {
                monster.room_id = self
                    .adventure
                    .rooms
                    .first()
                    .map(|room| room.id)
                    .unwrap_or(0);
            }
        }
        for shop in &mut self.adventure.shops {
            if shop.room_id == removed.id {
                shop.room_id = self
                    .adventure
                    .rooms
                    .first()
                    .map(|room| room.id)
                    .unwrap_or(0);
            }
        }
        if self.adventure.start_room == removed.id {
            self.adventure.start_room = self
                .adventure
                .rooms
                .first()
                .map(|room| room.id)
                .unwrap_or(0);
        }
        self.selected_room = if self.adventure.rooms.is_empty() {
            None
        } else {
            Some(idx.min(self.adventure.rooms.len() - 1))
        };
        self.status = format!("Room '{}' deleted", removed.name);
    }

    fn add_item(&mut self) {
        self.mark_modified();
        let id = self
            .adventure
            .items
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        self.adventure.items.push(ItemDef {
            id,
            name: format!("Item {}", id),
            description: "A new item".to_string(),
            location: self.adventure.start_room,
            ..ItemDef::default()
        });
        self.status = format!("Item {} added", id);
    }

    fn delete_item(&mut self) {
        let Some(idx) = self.selected_item else {
            return;
        };
        self.mark_modified();
        let removed = self.adventure.items.remove(idx);
        for room in &mut self.adventure.rooms {
            room.items.retain(|id| *id != removed.id);
        }
        for monster in &mut self.adventure.monsters {
            if monster.weapon_id == Some(removed.id) {
                monster.weapon_id = None;
            }
            monster.loot.retain(|id| *id != removed.id);
        }
        for shop in &mut self.adventure.shops {
            shop.stock.retain(|stock| stock.item_id != removed.id);
        }
        self.adventure
            .player_start_inventory
            .retain(|entry| !matches!(entry, InventoryRef::Id(id) if *id == removed.id));
        self.selected_item = if self.adventure.items.is_empty() {
            None
        } else {
            Some(idx.min(self.adventure.items.len() - 1))
        };
        self.status = format!("Item '{}' deleted", removed.name);
    }

    fn add_monster(&mut self) {
        self.mark_modified();
        let id = self
            .adventure
            .monsters
            .iter()
            .map(|monster| monster.id)
            .max()
            .unwrap_or(0)
            + 1;
        self.adventure.monsters.push(MonsterDef {
            id,
            name: format!("Monster {}", id),
            description: "A new monster".to_string(),
            room_id: self.adventure.start_room,
            friendliness: MonsterStatus::Neutral,
            ..MonsterDef::default()
        });
        self.status = format!("Monster {} added", id);
    }

    fn delete_monster(&mut self) {
        let Some(idx) = self.selected_monster else {
            return;
        };
        self.mark_modified();
        let removed = self.adventure.monsters.remove(idx);
        for item in &mut self.adventure.items {
            if item.location == removed.id {
                item.location = 0;
            }
        }
        self.selected_monster = if self.adventure.monsters.is_empty() {
            None
        } else {
            Some(idx.min(self.adventure.monsters.len() - 1))
        };
        self.status = format!("Monster '{}' deleted", removed.name);
    }

    fn add_quest(&mut self) {
        self.mark_modified();
        let id = self
            .adventure
            .quests
            .iter()
            .map(|quest| quest.id)
            .max()
            .unwrap_or(0)
            + 1;
        self.adventure.quests.push(QuestDef {
            id,
            title: format!("Quest {}", id),
            description: "A new quest".to_string(),
            objectives: vec![ObjectiveDef::Structured(StructuredObjective {
                kind: ObjectiveKind::Discover,
                description: "Complete objective 1".to_string(),
                target: Some("somewhere".to_string()),
                target_id: Some(serde_json::Value::String("somewhere".to_string())),
                count: 1,
                ..StructuredObjective::default()
            })],
            rewards: QuestRewardsDef {
                gold: 50,
                xp: 100,
                ..QuestRewardsDef::default()
            },
            ..QuestDef::default()
        });
        self.status = format!("Quest {} added", id);
    }

    fn delete_quest(&mut self) {
        let Some(idx) = self.selected_quest else {
            return;
        };
        self.mark_modified();
        let removed = self.adventure.quests.remove(idx);
        self.selected_quest = if self.adventure.quests.is_empty() {
            None
        } else {
            Some(idx.min(self.adventure.quests.len() - 1))
        };
        self.status = format!("Quest '{}' deleted", removed.title);
    }

    fn add_shop(&mut self) {
        self.mark_modified();
        let id = self
            .adventure
            .shops
            .iter()
            .map(|shop| shop.id)
            .max()
            .unwrap_or(0)
            + 1;
        self.adventure.shops.push(ShopDef {
            id,
            name: format!("Shop {}", id),
            room_id: self.adventure.start_room,
            gold: 200,
            ..ShopDef::default()
        });
        self.selected_shop = Some(self.adventure.shops.len() - 1);
        self.status = format!("Shop {} added", id);
    }

    fn delete_shop(&mut self) {
        let Some(idx) = self.selected_shop else {
            return;
        };
        self.mark_modified();
        let removed = self.adventure.shops.remove(idx);
        self.selected_shop = if self.adventure.shops.is_empty() {
            None
        } else {
            Some(idx.min(self.adventure.shops.len() - 1))
        };
        self.status = format!("Shop '{}' deleted", removed.name);
    }

    // Game operations
    fn start_game(&mut self) {
        self.game_output.clear();

        let mut adventure_game = AdventureGame::new("<editor>".to_string());
        adventure_game.add_system(Box::new(BasicWorldSystem));
        adventure_game.add_system(Box::new(InventorySystem));
        adventure_game.add_system(Box::new(CombatSystem));
        adventure_game.add_system(Box::new(QuestSystem::new()));
        adventure_game.add_system(Box::new(ShopSystem));

        match adventure_game.load_document(self.adventure.clone()) {
            Ok(intro) => {
                self.game_output.push(intro);
                self.game_output.push(adventure_game.look());
                self.game = Some(adventure_game);
                self.status = "Game started".to_string();
            }
            Err(e) => {
                self.game_output
                    .push(format!("Failed to load adventure: {e}"));
                self.status = "Failed to start game".to_string();
            }
        }
    }

    fn stop_game(&mut self) {
        self.game = None;
        self.game_output.clear();
        self.status = "Game stopped".to_string();
    }

    fn restart_game(&mut self) {
        self.stop_game();
        self.start_game();
    }

    fn send_game_command(&mut self) {
        if self.game_input.is_empty() {
            return;
        }
        let command = self.game_input.clone();
        self.game_input.clear();
        self.game_output.push(format!("> {}", command));

        match command.trim().to_lowercase().as_str() {
            "quit" | "q" | "exit" => {
                self.game_output.push("Game stopped.".to_string());
                self.game = None;
                self.status = "Game stopped".to_string();
                return;
            }
            _ => {}
        }

        if let Some(game) = &mut self.game {
            let lines = game.process_command(&command);
            self.game_output.extend(lines);
        } else {
            self.game_output
                .push("No game running. Press \u{25B6} Start Game first.".to_string());
        }
    }

    fn copy_json_to_clipboard(&mut self, ctx: &egui::Context) {
        match self.adventure.to_json_string() {
            Ok(json) => {
                ctx.copy_text(json);
                self.status = "JSON copied to clipboard".to_string();
            }
            Err(error) => {
                self.status = format!("Could not serialise JSON: {}", error);
            }
        }
    }

    fn show_json_diff(&mut self) {
        let current = self.adventure.to_json_string().unwrap_or_default();
        let saved = self.last_saved_json.clone().unwrap_or_default();
        if current == saved {
            self.diff_text = None;
            self.status = "No differences from the last saved version".to_string();
            return;
        }
        let mut diff = String::from("--- last saved\n+++ current\n");
        for (line_number, (old, new)) in saved.lines().zip(current.lines()).enumerate() {
            if old != new {
                diff.push_str(&format!(
                    "@@ line {} @@\n-{}\n+{}\n",
                    line_number + 1,
                    old,
                    new
                ));
            }
        }
        let saved_lines = saved.lines().count();
        let current_lines = current.lines().count();
        if saved_lines != current_lines {
            diff.push_str(&format!(
                "@@ line count: saved {} / current {} @@\n",
                saved_lines, current_lines
            ));
        }
        self.diff_text = Some(diff);
        self.status = "Diff shown".to_string();
    }

    fn generate_json_preview(&self) -> String {
        self.adventure
            .to_json_string()
            .unwrap_or_else(|error| format!("JSON serialisation error: {error}"))
    }
}
