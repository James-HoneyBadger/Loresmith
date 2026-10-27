use eframe::egui;
use loresmith_rs::{
    AdventureDocument, AdventureGame, BasicWorldSystem, CombatSystem, InventoryRef,
    InventorySystem, ItemDef, ItemType, MonsterDef, MonsterStatus, ObjectiveDef, ObjectiveKind,
    QuestDef, QuestRewardsDef, QuestSystem, RoomDef, ShopDef, ShopSystem, StructuredObjective,
};
use loresmith_ui as theme;
use std::path::PathBuf;

/// The GUI edits the canonical document directly so every field round-trips.
type AdventureData = AdventureDocument;

pub fn main_entry() -> eframe::Result<()> {
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

pub struct LoresmithStudio {
    pub(crate) adventure: AdventureData,
    pub(crate) current_file: Option<PathBuf>,
    pub(crate) modified: bool,
    pub(crate) active_tab: Tab,
    pub(crate) status: String,
    // Tab-specific state
    pub(crate) selected_room: Option<usize>,
    pub(crate) selected_item: Option<usize>,
    pub(crate) selected_monster: Option<usize>,
    pub(crate) selected_quest: Option<usize>,
    pub(crate) selected_shop: Option<usize>,
    pub(crate) room_filter: String,
    pub(crate) item_filter: String,
    pub(crate) monster_filter: String,
    pub(crate) quest_filter: String,
    pub(crate) shop_filter: String,
    // Play tab state
    pub(crate) game: Option<AdventureGame>,
    pub(crate) game_output: Vec<String>,
    pub(crate) game_input: String,
    // Exit confirmation
    pub(crate) show_exit_confirm: bool,
    // Add-exit dialog state
    pub(crate) new_exit_direction: String,
    pub(crate) new_exit_target: i32,
    // JSON preview / diff support
    pub(crate) last_saved_json: Option<String>,
    pub(crate) diff_text: Option<String>,
    // Undo / redo snapshots
    pub(crate) undo_stack: Vec<AdventureData>,
    pub(crate) redo_stack: Vec<AdventureData>,
    pub(crate) frame_snapshot: Option<AdventureData>,
    pub(crate) theme_mode: theme::ThemeMode,
    pub(crate) theme_applied: bool,
    pub(crate) classic_chrome: bool,
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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let storage = cc
            .storage
            .as_ref()
            .map(|storage| &**storage as &dyn eframe::Storage);
        Self::with_storage(storage)
    }

    /// Build the editor from optional persisted settings.
    pub fn with_storage(storage: Option<&dyn eframe::Storage>) -> Self {
        let mut app = Self::default();
        if let Some(storage) = storage
            && let Some(mode) = storage.get_string("theme")
        {
            app.theme_mode = theme::ThemeMode::from_key(&mode);
        }
        theme::set_mode(app.theme_mode);
        app
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum Tab {
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

impl Tab {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Tab::Play => "Play",
            Tab::Info => "Overview",
            Tab::Rooms => "Rooms",
            Tab::Items => "Items",
            Tab::Monsters => "Monsters",
            Tab::Quests => "Quests",
            Tab::Shops => "Shops",
            Tab::Preview => "Preview",
        }
    }
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

pub(crate) fn ensure_structured_objective(
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

pub(crate) fn objective_kind_label(kind: ObjectiveKind) -> &'static str {
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
pub(crate) struct GameSummary {
    pub(crate) room: String,
    pub(crate) health: i32,
    pub(crate) max_health: i32,
    pub(crate) level: i32,
    pub(crate) xp: i32,
    pub(crate) gold: i32,
    pub(crate) carried: usize,
    pub(crate) reputation: String,
    pub(crate) weapon: String,
    pub(crate) armor: String,
    pub(crate) inventory: Vec<String>,
    pub(crate) over_text: Option<String>,
}

impl eframe::App for LoresmithStudio {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("theme", self.theme_mode.key().to_string());
    }

    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.update_ui(root_ui);
    }
}

impl LoresmithStudio {
    /// Workspace tab titles, in navigation order.
    pub const TAB_TITLES: &'static [&'static str] = &[
        "Play", "Overview", "Rooms", "Items", "Monsters", "Quests", "Shops", "Preview",
    ];

    /// Activate a workspace tab by title. Returns false for an unknown title.
    pub fn select_tab(&mut self, title: &str) -> bool {
        let tab = match title {
            "Play" => Tab::Play,
            "Overview" => Tab::Info,
            "Rooms" => Tab::Rooms,
            "Items" => Tab::Items,
            "Monsters" => Tab::Monsters,
            "Quests" => Tab::Quests,
            "Shops" => Tab::Shops,
            "Preview" => Tab::Preview,
            _ => return false,
        };
        self.active_tab = tab;
        true
    }

    /// Title of the active workspace tab.
    pub fn active_tab_title(&self) -> &'static str {
        self.active_tab.title()
    }

    /// Whether the document has unsaved edits.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Render the whole editor UI into `root_ui`.
    pub fn update_ui(&mut self, root_ui: &mut egui::Ui) {
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
    pub(crate) fn show_caption_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
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

    pub(crate) fn show_toolbar(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn show_header(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn show_status_bar(&mut self, ui: &mut egui::Ui) {
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
                ui.label(egui::RichText::new("•").color(color).size(11.0));
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

    pub(crate) fn show_nav(&mut self, ui: &mut egui::Ui) {
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
        for (tab, subtitle) in [
            (Tab::Play, "Run the adventure"),
            (Tab::Info, "Identity and ending"),
            (Tab::Rooms, "Maps, exits and locks"),
            (Tab::Items, "Gear, loot and light"),
            (Tab::Monsters, "NPCs, dialogue and loot"),
            (Tab::Quests, "Goals, stages and rewards"),
            (Tab::Shops, "Stock and pricing"),
            (Tab::Preview, "JSON and validation"),
        ] {
            if theme::nav_button(ui, self.active_tab == tab, tab.title(), subtitle) {
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

    pub(crate) fn show_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
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

    pub(crate) fn show_main_ui(&mut self, ui: &mut egui::Ui) {
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

    // File operations
    pub(crate) fn new_adventure(&mut self) {
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

    pub(crate) fn open_adventure(&mut self) {
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

    pub(crate) fn save_adventure(&mut self) -> bool {
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

    pub(crate) fn save_adventure_as(&mut self) -> bool {
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

    pub(crate) fn save_to_file(
        &mut self,
        path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.adventure.save_json_file(path)?;
        self.last_saved_json = self.adventure.to_json_string().ok();
        self.frame_snapshot = None;
        Ok(())
    }

    pub(crate) fn load_from_file(
        &mut self,
        path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

    pub(crate) fn validate_adventure(&mut self) -> bool {
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

    pub(crate) fn export_to_json(&mut self) {
        self.save_adventure_as();
    }

    pub(crate) fn show_about(&mut self) {
        self.status = "Loresmith Studio — complete adventure editor".to_string();
    }

    pub(crate) fn mark_modified(&mut self) {
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

    pub(crate) fn undo(&mut self) {
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

    pub(crate) fn redo(&mut self) {
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
    pub(crate) fn add_room(&mut self) {
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

    pub(crate) fn delete_room(&mut self) {
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

    pub(crate) fn add_item(&mut self) {
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

    pub(crate) fn delete_item(&mut self) {
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

    pub(crate) fn add_monster(&mut self) {
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

    pub(crate) fn delete_monster(&mut self) {
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

    pub(crate) fn add_quest(&mut self) {
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

    pub(crate) fn delete_quest(&mut self) {
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

    pub(crate) fn add_shop(&mut self) {
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

    pub(crate) fn delete_shop(&mut self) {
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
    pub(crate) fn start_game(&mut self) {
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

    pub(crate) fn stop_game(&mut self) {
        self.game = None;
        self.game_output.clear();
        self.status = "Game stopped".to_string();
    }

    pub(crate) fn restart_game(&mut self) {
        self.stop_game();
        self.start_game();
    }

    pub(crate) fn send_game_command(&mut self) {
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

    pub(crate) fn copy_json_to_clipboard(&mut self, ctx: &egui::Context) {
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

    pub(crate) fn show_json_diff(&mut self) {
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

    pub(crate) fn generate_json_preview(&self) -> String {
        self.adventure
            .to_json_string()
            .unwrap_or_else(|error| format!("JSON serialisation error: {error}"))
    }
}
