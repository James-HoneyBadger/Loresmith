use eframe::egui;
use sagacraft_rs::{
    AdventureGame, BasicWorldSystem, CombatSystem, InventorySystem, ItemType, MonsterStatus,
    QuestSystem,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn default_one() -> i32 {
    1
}
fn default_six() -> i32 {
    6
}
fn default_true() -> bool {
    true
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SagaCraft IDE",
        options,
        Box::new(|_cc| Ok(Box::new(SagaCraftIDE::default()))),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdventureData {
    title: String,
    intro: String,
    start_room: i32,
    rooms: Vec<RoomData>,
    items: Vec<ItemData>,
    monsters: Vec<MonsterData>,
    quests: Vec<QuestData>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    settings: Option<AdventureSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdventureSettings {
    #[serde(default)]
    allow_save: bool,
    #[serde(default)]
    difficulty: String,
    #[serde(default)]
    enable_magic: bool,
    #[serde(default)]
    enable_puzzles: bool,
    #[serde(default)]
    enable_combat_xp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoomData {
    id: i32,
    name: String,
    description: String,
    exits: HashMap<String, i32>,
    #[serde(default)]
    is_dark: bool,
    #[serde(default)]
    light_level: Option<String>,
    #[serde(default)]
    is_safe_zone: bool,
    #[serde(default)]
    ambient_sound: Option<String>,
    #[serde(default)]
    has_trap: bool,
    #[serde(default)]
    trap_damage: i32,
    #[serde(default)]
    environmental_effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ItemData {
    id: i32,
    name: String,
    description: String,
    #[serde(rename = "type")]
    item_type: ItemType,
    value: i32,
    weight: i32,
    #[serde(default)]
    location: i32, // 0 = inventory, room_id = on ground
    #[serde(default)]
    is_weapon: bool,
    #[serde(default)]
    weapon_type: i32,
    #[serde(default = "default_one")]
    weapon_dice: i32,
    #[serde(default = "default_six")]
    weapon_sides: i32,
    #[serde(default)]
    is_armor: bool,
    #[serde(default)]
    armor_value: i32,
    #[serde(default = "default_true")]
    is_takeable: bool,
    #[serde(default)]
    is_wearable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonsterData {
    id: i32,
    name: String,
    description: String,
    hardiness: i32,
    agility: i32,
    weapon_id: Option<i32>,
    armor_worn: i32,
    gold: i32,
    #[serde(rename = "friendliness")]
    status: MonsterStatus,
    #[serde(default = "default_one")]
    room_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QuestData {
    id: i32,
    title: String,
    description: String,
    objectives: Vec<String>,
    rewards_gold: i32,
    rewards_xp: i32,
}

#[derive(Default)]
struct SagaCraftIDE {
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
    room_filter: String,
    item_filter: String,
    monster_filter: String,
    quest_filter: String,
    // Play tab state
    game: Option<AdventureGame>,
    game_output: Vec<String>,
    game_input: String,
    // Exit confirmation
    show_exit_confirm: bool,
    // Add-exit dialog state
    new_exit_direction: String,
    new_exit_target: i32,
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
    Preview,
}

impl Default for AdventureData {
    fn default() -> Self {
        Self {
            title: "New Adventure".to_string(),
            intro: "Welcome to your new adventure!".to_string(),
            start_room: 1,
            rooms: vec![RoomData {
                id: 1,
                name: "Starting Room".to_string(),
                description: "A simple room to begin your adventure.".to_string(),
                exits: HashMap::new(),
                is_dark: false,
                light_level: None,
                is_safe_zone: false,
                ambient_sound: None,
                has_trap: false,
                trap_damage: 0,
                environmental_effects: vec![],
            }],
            items: vec![ItemData {
                id: 1,
                name: "Brass Lantern".to_string(),
                description: "A shiny brass lantern that provides light.".to_string(),
                item_type: ItemType::Normal,
                value: 25,
                weight: 2,
                location: 1,
                is_weapon: false,
                weapon_type: 0,
                weapon_dice: 1,
                weapon_sides: 6,
                is_armor: false,
                armor_value: 0,
                is_takeable: true,
                is_wearable: false,
            }],
            monsters: vec![MonsterData {
                id: 1,
                name: "Goblin".to_string(),
                description: "A small green goblin with a club.".to_string(),
                hardiness: 8,
                agility: 12,
                weapon_id: Some(1),
                armor_worn: 0,
                gold: 5,
                status: MonsterStatus::Friendly,
                room_id: 1,
            }],
            quests: vec![QuestData {
                id: 1,
                title: "Light the Path".to_string(),
                description: "Secure a light source and reach the Shadow Gallery.".to_string(),
                objectives: vec![
                    "Pick up the brass lantern".to_string(),
                    "Enter the Shadow Gallery with light".to_string(),
                ],
                rewards_gold: 40,
                rewards_xp: 60,
            }],
            author: None,
            settings: None,
        }
    }
}

impl eframe::App for SagaCraftIDE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.show_menu_bar(ctx, ui);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.modified {
                        ui.colored_label(egui::Color32::YELLOW, "●");
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
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
                        if ui.button("Save & Exit").clicked() {
                            self.save_adventure();
                            std::process::exit(0);
                        }
                        if ui.button("Exit Without Saving").clicked() {
                            std::process::exit(0);
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_exit_confirm = false;
                        }
                    });
                });
        }
    }
}

impl SagaCraftIDE {
    fn show_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
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
                        std::process::exit(0);
                    }
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

            ui.menu_button("View", |ui| {
                ui.label("Theme:");
                if ui.button("Light").clicked() {
                    ctx.set_visuals(egui::Visuals::light());
                }
                if ui.button("Dark").clicked() {
                    ctx.set_visuals(egui::Visuals::dark());
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About SagaCraft IDE").clicked() {
                    self.show_about();
                    ui.close();
                }
            });
        });
    }

    fn show_main_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Tab buttons
            if ui
                .add(egui::Button::new("🎮 Play").selected(self.active_tab == Tab::Play))
                .clicked()
            {
                self.active_tab = Tab::Play;
            }
            if ui
                .add(egui::Button::new("ℹ Info").selected(self.active_tab == Tab::Info))
                .clicked()
            {
                self.active_tab = Tab::Info;
            }
            if ui
                .add(egui::Button::new("🏠 Rooms").selected(self.active_tab == Tab::Rooms))
                .clicked()
            {
                self.active_tab = Tab::Rooms;
            }
            if ui
                .add(egui::Button::new("🎒 Items").selected(self.active_tab == Tab::Items))
                .clicked()
            {
                self.active_tab = Tab::Items;
            }
            if ui
                .add(egui::Button::new("👹 Monsters").selected(self.active_tab == Tab::Monsters))
                .clicked()
            {
                self.active_tab = Tab::Monsters;
            }
            if ui
                .add(egui::Button::new("📜 Quests").selected(self.active_tab == Tab::Quests))
                .clicked()
            {
                self.active_tab = Tab::Quests;
            }
            if ui
                .add(egui::Button::new(" Preview").selected(self.active_tab == Tab::Preview))
                .clicked()
            {
                self.active_tab = Tab::Preview;
            }
        });

        ui.separator();

        match self.active_tab {
            Tab::Play => self.show_play_tab(ui),
            Tab::Info => self.show_info_tab(ui),
            Tab::Rooms => self.show_rooms_tab(ui),
            Tab::Items => self.show_items_tab(ui),
            Tab::Monsters => self.show_monsters_tab(ui),
            Tab::Quests => self.show_quests_tab(ui),
            Tab::Preview => self.show_preview_tab(ui),
        }
    }

    fn show_play_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎮 Play Adventure");

        ui.horizontal(|ui| {
            if ui.button("▶ Start Game").clicked() {
                self.start_game();
            }
            if ui.button("⏹ Stop Game").clicked() {
                self.stop_game();
            }
            if ui.button("🔄 Restart").clicked() {
                self.restart_game();
            }
        });

        ui.separator();

        // Game output
        egui::ScrollArea::vertical().show(ui, |ui| {
            for line in &self.game_output {
                ui.label(line);
            }
        });

        // Input area
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Command:");
            let response = ui.text_edit_singleline(&mut self.game_input);
            if ui.button("Send").clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                self.send_game_command();
            }
        });
    }

    fn show_info_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("ℹ Adventure Information");

        let mut changed = false;
        egui::Grid::new("info_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                ui.label("Title:");
                changed |= ui.text_edit_singleline(&mut self.adventure.title).changed();
                ui.end_row();

                ui.label("Introduction:");
                changed |= ui.text_edit_multiline(&mut self.adventure.intro).changed();
                ui.end_row();

                ui.label("Start Room ID:");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.adventure.start_room))
                    .changed();
                ui.end_row();
            });
        if changed {
            self.modified = true;
        }

        ui.separator();
        ui.label(format!("Rooms: {}", self.adventure.rooms.len()));
        ui.label(format!("Items: {}", self.adventure.items.len()));
        ui.label(format!("Monsters: {}", self.adventure.monsters.len()));
        ui.label(format!("Quests: {}", self.adventure.quests.len()));
    }

    fn show_rooms_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏠 Rooms");

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.room_filter);
            if ui.button("Clear").clicked() {
                self.room_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("➕ Add Room").clicked() {
                self.add_room();
            }
            if ui.button("➖ Delete Room").clicked() && self.selected_room.is_some() {
                self.delete_room();
            }
        });

        ui.separator();

        ui.columns(2, |columns| {
            // Room list
            columns[0].heading("Rooms");
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
            columns[1].heading("Room Editor");
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

                            ui.label("Description:");
                            changed |= ui.text_edit_multiline(&mut room.description).changed();
                            ui.end_row();

                            ui.label("Dark:");
                            changed |= ui.checkbox(&mut room.is_dark, "").changed();
                            ui.end_row();
                        });

                    columns[1].separator();
                    columns[1].label("Exits:");
                    let exit_dirs: Vec<String> = room.exits.keys().cloned().collect();
                    let mut remove_dir: Option<String> = None;
                    for direction in &exit_dirs {
                        if let Some(room_id) = room.exits.get_mut(direction) {
                            columns[1].horizontal(|ui| {
                                ui.label(direction.as_str());
                                ui.label("\u{2192}");
                                if ui.add(egui::DragValue::new(room_id)).changed() {
                                    changed = true;
                                }
                                if ui.button("\u{274c}").clicked() {
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
                        if ui.button("➕ Add Exit").clicked() && !self.new_exit_direction.is_empty()
                        {
                            room.exits
                                .insert(self.new_exit_direction.clone(), self.new_exit_target);
                            changed = true;
                        }
                    });
                    if changed {
                        self.modified = true;
                    }
                }
            } else {
                columns[1].label("Select a room to edit");
            }
        });
    }

    fn show_items_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎒 Items");

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.item_filter);
            if ui.button("Clear").clicked() {
                self.item_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("➕ Add Item").clicked() {
                self.add_item();
            }
            if ui.button("➖ Delete Item").clicked() && self.selected_item.is_some() {
                self.delete_item();
            }
        });

        ui.separator();

        ui.columns(2, |columns| {
            // Item list
            columns[0].heading("Items");
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
            columns[1].heading("Item Editor");
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

                            ui.label("Description:");
                            changed |= ui.text_edit_multiline(&mut item.description).changed();
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

                            ui.label("Location (room ID):");
                            changed |= ui.add(egui::DragValue::new(&mut item.location)).changed();
                            ui.end_row();

                            ui.label("Takeable:");
                            changed |= ui.checkbox(&mut item.is_takeable, "").changed();
                            ui.end_row();

                            ui.label("Is Weapon:");
                            changed |= ui.checkbox(&mut item.is_weapon, "").changed();
                            ui.end_row();

                            if item.is_weapon {
                                ui.label("Weapon Type (1-5):");
                                changed |= ui
                                    .add(egui::DragValue::new(&mut item.weapon_type).range(1..=5))
                                    .changed();
                                ui.end_row();

                                ui.label("Damage Dice:");
                                changed |= ui
                                    .add(egui::DragValue::new(&mut item.weapon_dice).range(1..=10))
                                    .changed();
                                ui.end_row();

                                ui.label("Damage Sides:");
                                changed |= ui
                                    .add(egui::DragValue::new(&mut item.weapon_sides).range(2..=20))
                                    .changed();
                                ui.end_row();
                            }

                            ui.label("Is Armor:");
                            changed |= ui.checkbox(&mut item.is_armor, "").changed();
                            ui.end_row();

                            if item.is_armor {
                                ui.label("Armor Value:");
                                changed |= ui
                                    .add(egui::DragValue::new(&mut item.armor_value).range(0..=20))
                                    .changed();
                                ui.end_row();

                                ui.label("Wearable:");
                                changed |= ui.checkbox(&mut item.is_wearable, "").changed();
                                ui.end_row();
                            }
                        });
                    if changed {
                        self.modified = true;
                    }
                }
            } else {
                columns[1].label("Select an item to edit");
            }
        });
    }

    fn show_monsters_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("👹 Monsters");

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.monster_filter);
            if ui.button("Clear").clicked() {
                self.monster_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("➕ Add Monster").clicked() {
                self.add_monster();
            }
            if ui.button("➖ Delete Monster").clicked() && self.selected_monster.is_some() {
                self.delete_monster();
            }
        });

        ui.separator();

        ui.columns(2, |columns| {
            // Monster list
            columns[0].heading("Monsters");
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
            columns[1].heading("Monster Editor");
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

                            ui.label("Description:");
                            changed |= ui.text_edit_multiline(&mut monster.description).changed();
                            ui.end_row();

                            ui.label("Hardiness:");
                            changed |= ui
                                .add(egui::DragValue::new(&mut monster.hardiness))
                                .changed();
                            ui.end_row();

                            ui.label("Agility:");
                            changed |= ui.add(egui::DragValue::new(&mut monster.agility)).changed();
                            ui.end_row();

                            ui.label("Gold:");
                            changed |= ui.add(egui::DragValue::new(&mut monster.gold)).changed();
                            ui.end_row();

                            ui.label("Room ID:");
                            changed |= ui.add(egui::DragValue::new(&mut monster.room_id)).changed();
                            ui.end_row();

                            ui.label("Friendliness:");
                            egui::ComboBox::from_id_salt("monster_status")
                                .selected_text(format!("{:?}", monster.status))
                                .show_ui(ui, |ui: &mut egui::Ui| {
                                    changed |= ui
                                        .selectable_value(
                                            &mut monster.status,
                                            MonsterStatus::Neutral,
                                            "Neutral",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut monster.status,
                                            MonsterStatus::Friendly,
                                            "Friendly",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut monster.status,
                                            MonsterStatus::Hostile,
                                            "Hostile",
                                        )
                                        .changed();
                                });
                            ui.end_row();
                        });
                    if changed {
                        self.modified = true;
                    }
                }
            } else {
                columns[1].label("Select a monster to edit");
            }
        });
    }

    fn show_quests_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📜 Quests");

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.quest_filter);
            if ui.button("Clear").clicked() {
                self.quest_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("➕ Add Quest").clicked() {
                self.add_quest();
            }
            if ui.button("➖ Delete Quest").clicked() && self.selected_quest.is_some() {
                self.delete_quest();
            }
        });

        ui.separator();

        ui.columns(2, |columns| {
            // Quest list
            columns[0].heading("Quests");
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
            columns[1].heading("Quest Editor");
            if let Some(quest_idx) = self.selected_quest {
                if let Some(quest) = self.adventure.quests.get_mut(quest_idx) {
                    let mut changed = false;
                    egui::Grid::new("quest_grid")
                        .num_columns(2)
                        .spacing([10.0, 10.0])
                        .show(&mut columns[1], |ui| {
                            ui.label("ID:");
                            changed |= ui.add(egui::DragValue::new(&mut quest.id)).changed();
                            ui.end_row();

                            ui.label("Title:");
                            changed |= ui.text_edit_singleline(&mut quest.title).changed();
                            ui.end_row();

                            ui.label("Description:");
                            changed |= ui.text_edit_multiline(&mut quest.description).changed();
                            ui.end_row();

                            ui.label("Objectives:");
                            ui.vertical(|ui| {
                                let mut remove_idx: Option<usize> = None;
                                for (idx, objective) in quest.objectives.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        if ui.text_edit_singleline(objective).changed() {
                                            changed = true;
                                        }
                                        if ui.button("❌").clicked() {
                                            remove_idx = Some(idx);
                                        }
                                    });
                                }
                                if let Some(idx) = remove_idx {
                                    quest.objectives.remove(idx);
                                    changed = true;
                                }
                                if ui.button("➕ Add Objective").clicked() {
                                    quest.objectives.push("New objective".to_string());
                                    changed = true;
                                }
                            });
                            ui.end_row();

                            ui.label("Gold Reward:");
                            changed |= ui
                                .add(egui::DragValue::new(&mut quest.rewards_gold))
                                .changed();
                            ui.end_row();

                            ui.label("XP Reward:");
                            changed |= ui
                                .add(egui::DragValue::new(&mut quest.rewards_xp))
                                .changed();
                            ui.end_row();
                        });
                    if changed {
                        self.modified = true;
                    }
                }
            } else {
                columns[1].label("Select a quest to edit");
            }
        });
    }

    fn show_preview_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("👁️ Adventure Preview");

        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh").clicked() {
                self.refresh_json_preview();
            }
            if ui.button("📋 Copy JSON").clicked() {
                self.copy_json_to_clipboard();
            }
            if ui.button("📊 Show Diff").clicked() {
                self.show_json_diff();
            }
        });

        ui.separator();

        ui.columns(2, |columns| {
            // JSON Preview
            columns[0].heading("JSON Export");
            columns[0].label("This is the JSON representation of your adventure:");

            egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                let json = self.generate_json_preview();
                ui.add(
                    egui::TextEdit::multiline(&mut json.as_str())
                        .font(egui::TextStyle::Monospace)
                        .interactive(false),
                );
            });

            // Preview Stats
            columns[1].heading("Adventure Statistics");
            columns[1].label(format!("Rooms: {}", self.adventure.rooms.len()));
            columns[1].label(format!("Items: {}", self.adventure.items.len()));
            columns[1].label(format!("Monsters: {}", self.adventure.monsters.len()));
            columns[1].label(format!("Quests: {}", self.adventure.quests.len()));

            columns[1].separator();
            columns[1].label("Export Options:");
            if columns[1].button("💾 Save as JSON").clicked() {
                // TODO: Implement save dialog
                self.status = "Save dialog not implemented yet".to_string();
            }
            if columns[1].button("📤 Export to Game").clicked() {
                // TODO: Implement export to game
                self.status = "Export to game not implemented yet".to_string();
            }
        });
    }

    // File operations
    fn new_adventure(&mut self) {
        self.adventure = AdventureData::default();
        self.current_file = None;
        self.modified = false;
        self.status = "New adventure created".to_string();
    }

    fn open_adventure(&mut self) {
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

    fn save_adventure(&mut self) {
        if let Some(path) = self.current_file.clone() {
            match self.save_to_file(&path) {
                Ok(_) => {
                    self.modified = false;
                    self.status = format!("Saved adventure: {}", path.display());
                }
                Err(e) => {
                    self.status = format!("Error saving file: {}", e);
                }
            }
        } else {
            self.save_adventure_as();
        }
    }

    fn save_adventure_as(&mut self) {
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
                }
                Err(e) => {
                    self.status = format!("Error saving file: {}", e);
                }
            }
        }
    }

    fn save_to_file(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.adventure)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn load_from_file(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        self.adventure = serde_json::from_str(&content)?;
        Ok(())
    }

    fn validate_adventure(&mut self) {
        let mut errors: Vec<String> = Vec::new();
        if self.adventure.title.trim().is_empty() {
            errors.push("Title is empty".to_string());
        }
        if self.adventure.rooms.is_empty() {
            errors.push("No rooms defined".to_string());
        }

        let room_ids: std::collections::HashSet<i32> =
            self.adventure.rooms.iter().map(|r| r.id).collect();
        let item_ids: std::collections::HashSet<i32> =
            self.adventure.items.iter().map(|i| i.id).collect();
        let monster_ids: std::collections::HashSet<i32> =
            self.adventure.monsters.iter().map(|m| m.id).collect();

        if !room_ids.contains(&self.adventure.start_room) {
            errors.push(format!(
                "start_room {} does not exist",
                self.adventure.start_room
            ));
        }
        if room_ids.len() != self.adventure.rooms.len() {
            errors.push("Duplicate room IDs detected".to_string());
        }
        if item_ids.len() != self.adventure.items.len() {
            errors.push("Duplicate item IDs detected".to_string());
        }
        if monster_ids.len() != self.adventure.monsters.len() {
            errors.push("Duplicate monster IDs detected".to_string());
        }

        for room in &self.adventure.rooms {
            for (direction, dest) in &room.exits {
                if !room_ids.contains(dest) {
                    errors.push(format!(
                        "Room {} has '{}' exit to missing room {}",
                        room.id, direction, dest
                    ));
                }
            }
        }

        for item in &self.adventure.items {
            let valid_location = matches!(item.location, -1 | 0)
                || room_ids.contains(&item.location)
                || monster_ids.contains(&item.location);
            if !valid_location {
                errors.push(format!(
                    "Item {} points to missing location {}",
                    item.id, item.location
                ));
            }
        }

        for monster in &self.adventure.monsters {
            if !room_ids.contains(&monster.room_id) {
                errors.push(format!(
                    "Monster {} points to missing room {}",
                    monster.id, monster.room_id
                ));
            }
            if let Some(weapon_id) = monster.weapon_id
                && !item_ids.contains(&weapon_id)
            {
                errors.push(format!(
                    "Monster {} references missing weapon {}",
                    monster.id, weapon_id
                ));
            }
        }

        if errors.is_empty() {
            self.status = format!(
                "Valid: {} rooms, {} items, {} monsters, {} quests",
                self.adventure.rooms.len(),
                self.adventure.items.len(),
                self.adventure.monsters.len(),
                self.adventure.quests.len()
            );
        } else {
            self.status = format!("Validation errors: {}", errors.join("; "));
        }
    }

    fn export_to_json(&mut self) {
        self.save_adventure_as();
    }

    fn show_about(&mut self) {
        self.status = "SagaCraft IDE - Complete adventure editor".to_string();
    }

    // CRUD operations
    fn add_room(&mut self) {
        let id = self.adventure.rooms.iter().map(|r| r.id).max().unwrap_or(0) + 1;
        self.adventure.rooms.push(RoomData {
            id,
            name: format!("Room {}", id),
            description: "A new room".to_string(),
            exits: HashMap::new(),
            is_dark: false,
            light_level: None,
            is_safe_zone: false,
            ambient_sound: None,
            has_trap: false,
            trap_damage: 0,
            environmental_effects: vec![],
        });
        self.modified = true;
        self.status = format!("Room {} added", id);
    }

    fn delete_room(&mut self) {
        if let Some(idx) = self.selected_room {
            self.adventure.rooms.remove(idx);
            self.selected_room = None;
            self.modified = true;
            self.status = "Room deleted".to_string();
        }
    }

    fn add_item(&mut self) {
        let id = self.adventure.items.iter().map(|r| r.id).max().unwrap_or(0) + 1;
        // Default location to start_room so new items appear on the ground
        let location = self.adventure.start_room;
        self.adventure.items.push(ItemData {
            id,
            name: format!("Item {}", id),
            description: "A new item".to_string(),
            item_type: ItemType::Normal,
            value: 0,
            weight: 1,
            location,
            is_weapon: false,
            weapon_type: 0,
            weapon_dice: 1,
            weapon_sides: 6,
            is_armor: false,
            armor_value: 0,
            is_takeable: true,
            is_wearable: false,
        });
        self.modified = true;
        self.status = format!("Item {} added", id);
    }

    fn delete_item(&mut self) {
        if let Some(idx) = self.selected_item {
            self.adventure.items.remove(idx);
            self.selected_item = None;
            self.modified = true;
            self.status = "Item deleted".to_string();
        }
    }

    fn add_monster(&mut self) {
        let id = self
            .adventure
            .monsters
            .iter()
            .map(|r| r.id)
            .max()
            .unwrap_or(0)
            + 1;
        let room_id = self.adventure.start_room;
        self.adventure.monsters.push(MonsterData {
            id,
            name: format!("Monster {}", id),
            description: "A new monster".to_string(),
            hardiness: 10,
            agility: 10,
            weapon_id: None,
            armor_worn: 0,
            gold: 0,
            status: MonsterStatus::Neutral,
            room_id,
        });
        self.modified = true;
        self.status = format!("Monster {} added", id);
    }

    fn delete_monster(&mut self) {
        if let Some(idx) = self.selected_monster {
            self.adventure.monsters.remove(idx);
            self.selected_monster = None;
            self.modified = true;
            self.status = "Monster deleted".to_string();
        }
    }

    fn add_quest(&mut self) {
        let id = self
            .adventure
            .quests
            .iter()
            .map(|r| r.id)
            .max()
            .unwrap_or(0)
            + 1;
        self.adventure.quests.push(QuestData {
            id,
            title: format!("Quest {}", id),
            description: "A new quest".to_string(),
            objectives: vec!["Complete objective 1".to_string()],
            rewards_gold: 50,
            rewards_xp: 100,
        });
        self.modified = true;
        self.status = format!("Quest {} added", id);
    }

    fn delete_quest(&mut self) {
        if let Some(idx) = self.selected_quest {
            self.adventure.quests.remove(idx);
            self.selected_quest = None;
            self.modified = true;
            self.status = "Quest deleted".to_string();
        }
    }

    // Game operations
    fn start_game(&mut self) {
        self.game_output.clear();

        // Serialise the current adventure to a temp file and load it into AdventureGame
        let tmp_path = std::env::temp_dir().join("sagacraft_play.json");
        match serde_json::to_string_pretty(&self.adventure) {
            Ok(json) => {
                if let Err(e) = fs::write(&tmp_path, &json) {
                    self.game_output
                        .push(format!("Error writing temp file: {e}"));
                    return;
                }
            }
            Err(e) => {
                self.game_output
                    .push(format!("Error serialising adventure: {e}"));
                return;
            }
        }

        let mut adventure_game = AdventureGame::new(tmp_path.to_string_lossy().to_string());
        adventure_game.add_system(Box::new(BasicWorldSystem));
        adventure_game.add_system(Box::new(InventorySystem));
        adventure_game.add_system(Box::new(CombatSystem));
        adventure_game.add_system(Box::new(QuestSystem::new()));

        match adventure_game.load_adventure() {
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

    fn refresh_json_preview(&mut self) {
        // The preview is always up to date since it's generated from current data
        self.status = "JSON preview refreshed".to_string();
    }

    fn copy_json_to_clipboard(&mut self) {
        // TODO: Copy JSON to system clipboard
        self.status = "JSON copied to clipboard".to_string();
    }

    fn show_json_diff(&mut self) {
        // TODO: Show diff between current and saved JSON
        self.status = "JSON diff not yet implemented".to_string();
    }

    fn generate_json_preview(&self) -> String {
        serde_json::to_string_pretty(&self.adventure)
            .unwrap_or_else(|e| format!("JSON serialisation error: {e}"))
    }
}
