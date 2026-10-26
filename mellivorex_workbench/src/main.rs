use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use mellivorex_rs::{
    AdventureDocument, ItemType, MonsterStatus, ObjectiveDef, ObjectiveKind, QuestDef,
    QuestRewardsDef, RoomDef, StageDef, StructuredObjective,
};

fn main() -> anyhow::Result<()> {
    let args = Args::parse(std::env::args().skip(1).collect());
    if args.show_help {
        print_help();
        return Ok(());
    }

    let file_path = args
        .file
        .clone()
        .or_else(|| args.new.clone())
        .unwrap_or_else(|| PathBuf::from("examples/adventures/demo_adventure.json"));

    let adventure = if args.new.is_some() {
        if file_path.exists() && !args.force {
            eprintln!(
                "Refusing to overwrite existing file '{}'. Use --force to replace it.",
                file_path.display()
            );
            std::process::exit(1);
        }
        starter_document()
    } else {
        match AdventureDocument::load_json_file(&file_path) {
            Ok(document) => document,
            Err(error) => {
                eprintln!(
                    "Failed to load '{}': {}. The file was not modified.",
                    file_path.display(),
                    error
                );
                std::process::exit(1);
            }
        }
    };

    let mut app = App::new(file_path, adventure);
    let mut tui = Tui::new()?;
    let result = run(&mut tui, &mut app);
    tui.shutdown()?;
    result
}

fn starter_document() -> AdventureDocument {
    let mut document = AdventureDocument {
        id: "new_adventure".to_string(),
        title: "New Adventure".to_string(),
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
    document
}

fn print_help() {
    println!("Mellivorex Workbench");
    println!("Usage:");
    println!("  mellivorex_workbench --file <path>");
    println!("  mellivorex_workbench --new <path> [--force]");
    println!();
    println!("Keys:");
    println!("  Up/Down   select room");
    println!("  :         command mode");
    println!("  Esc       normal mode");
    println!("  q         quit (asks if unsaved)");
    println!("  s         save");
    println!();
    println!("Commands (type after ':' then Enter):");
    println!("  w | write                    save");
    println!("  q | quit                     quit (asks if unsaved)");
    println!("  q!                           quit without saving");
    println!("  wq                           save then quit");
    println!("  validate                     validate the document");
    println!("  set start <room_id>          set start room");
    println!("  room add <id>                add room");
    println!("  room del [id]                delete room (default selected)");
    println!("  room set title|desc <text>   edit selected room");
    println!("  exit set <dir> <dest>        set selected room exit");
    println!("  exit lock <dir> <key>        lock selected room exit");
    println!("  exit unlock <dir>            unlock selected room exit");
    println!("  exit del <dir>               delete selected room exit");
    println!("  item add <id> <name> <desc>  add item to selected room");
    println!("  item del <id|name>           delete item");
    println!("  item set <id> name|desc|type|value|weight|location|light <value>");
    println!("  item weapon <id> <dice> <sides>   make item a weapon");
    println!("  monster add <id> <room> <name>    add monster");
    println!("  monster del <id>             delete monster");
    println!("  monster set <id> name|desc|room|hp|agility|gold|status <value>");
    println!("  quest add <id> <title>       add quest");
    println!("  quest del <id>               delete quest");
    println!("  quest objective <id> <kill|collect|reach|talk> <target> [count]");
    println!("  shop add <id> <room> <name>  add shop");
    println!("  shop stock <shop_id> <item_id> [qty]");
    println!("  shop del <id>                delete shop");
    println!();
    println!("Tip: quote values with spaces, e.g. room set title \"Grand Hall\".");
}

#[derive(Debug, Default)]
struct Args {
    file: Option<PathBuf>,
    new: Option<PathBuf>,
    force: bool,
    show_help: bool,
}

impl Args {
    fn parse(mut argv: Vec<String>) -> Self {
        let mut out = Args::default();
        while let Some(arg) = argv.first().cloned() {
            argv.remove(0);
            match arg.as_str() {
                "--help" | "-h" => {
                    out.show_help = true;
                    break;
                }
                "--force" => out.force = true,
                "--file" | "-f" => {
                    if let Some(path) = argv.first().cloned() {
                        argv.remove(0);
                        out.file = Some(PathBuf::from(path));
                    } else {
                        out.show_help = true;
                        break;
                    }
                }
                "--new" => {
                    if let Some(path) = argv.first().cloned() {
                        argv.remove(0);
                        out.new = Some(PathBuf::from(path));
                    } else {
                        out.show_help = true;
                        break;
                    }
                }
                other => {
                    if !other.starts_with('-') && out.file.is_none() {
                        out.file = Some(PathBuf::from(other));
                    } else {
                        eprintln!("Unknown argument: {}", other);
                        out.show_help = true;
                        break;
                    }
                }
            }
        }
        out
    }
}

struct Tui {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl Tui {
    fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        Ok(Self {
            terminal: Terminal::new(backend)?,
        })
    }

    fn shutdown(&mut self) -> anyhow::Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Command,
}

struct App {
    file: PathBuf,
    adventure: AdventureDocument,
    selected_room: usize,
    mode: Mode,
    cmd: String,
    status: String,
    dirty: bool,
    quit_confirm: bool,
    force_quit: bool,
}

impl App {
    fn new(file: PathBuf, adventure: AdventureDocument) -> Self {
        Self {
            file,
            adventure,
            selected_room: 0,
            mode: Mode::Normal,
            cmd: String::new(),
            status: "Press ':' for commands. 's' to save, 'q' to quit.".to_string(),
            dirty: false,
            quit_confirm: false,
            force_quit: false,
        }
    }

    fn selected_room_mut(&mut self) -> Option<&mut RoomDef> {
        self.adventure.rooms.get_mut(self.selected_room)
    }

    fn selected_room(&self) -> Option<&RoomDef> {
        self.adventure.rooms.get(self.selected_room)
    }

    fn clamp_selection(&mut self) {
        if self.adventure.rooms.is_empty() {
            self.selected_room = 0;
        } else if self.selected_room >= self.adventure.rooms.len() {
            self.selected_room = self.adventure.rooms.len() - 1;
        }
    }

    fn save(&mut self) -> bool {
        match self.adventure.save_json_file(&self.file) {
            Ok(()) => {
                self.status = format!("Saved {}", self.file.display());
                self.dirty = false;
                self.quit_confirm = false;
                true
            }
            Err(error) => {
                self.status = format!("Save failed: {error}");
                false
            }
        }
    }

    fn validate(&mut self) {
        let report = self.adventure.report();
        if report.is_valid() {
            self.status = if report.warnings.is_empty() {
                "Document is valid.".to_string()
            } else {
                format!(
                    "Valid with {} warning(s): {}",
                    report.warnings.len(),
                    report.warnings.join("; ")
                )
            };
        } else {
            self.status = format!(
                "Validation errors: {}{}",
                report.errors.join("; "),
                if report.warnings.is_empty() {
                    String::new()
                } else {
                    format!("; warnings: {}", report.warnings.join("; "))
                }
            );
        }
    }

    fn exec_command(&mut self, raw: &str) {
        let line = raw.trim();
        if line.is_empty() {
            return;
        }

        let words = match parse_words(line) {
            Ok(words) => words,
            Err(message) => {
                self.status = format!("Command parse error: {message}");
                return;
            }
        };
        let Some(command) = words.first().map(String::as_str) else {
            return;
        };

        match command {
            "help" => {
                self.status =
                    "Commands: w, q, q!, wq, validate, set, room, exit, item, monster, quest, shop"
                        .to_string();
            }
            "w" | "write" => {
                self.save();
            }
            "q" | "quit" => {
                if self.dirty {
                    self.status =
                        "Unsaved changes. Use ':w' to save, ':q!' to discard, or press q again."
                            .to_string();
                    self.quit_confirm = true;
                } else {
                    self.force_quit = true;
                }
            }
            "q!" => {
                self.force_quit = true;
            }
            "wq" => {
                if self.save() {
                    self.force_quit = true;
                }
            }
            "validate" => self.validate(),
            "set" => {
                if words.get(1).map(String::as_str) == Some("start") {
                    if let Some(room_id) = words.get(2).and_then(|id| id.parse::<i32>().ok()) {
                        if self.adventure.rooms.iter().any(|room| room.id == room_id) {
                            self.adventure.start_room = room_id;
                            self.dirty = true;
                            self.status = format!("start_room set to {room_id}");
                        } else {
                            self.status = format!("No such room: {room_id}");
                        }
                    } else {
                        self.status = "usage: set start <room_id>".to_string();
                    }
                } else {
                    self.status = "usage: set start <room_id>".to_string();
                }
            }
            "room" => self.exec_room_command(&words),
            "exit" => self.exec_exit_command(&words),
            "item" => self.exec_item_command(&words),
            "monster" => self.exec_monster_command(&words),
            "quest" => self.exec_quest_command(&words),
            "shop" => self.exec_shop_command(&words),
            other => {
                self.status = format!("Unknown command: {other}");
            }
        }
    }

    fn exec_room_command(&mut self, words: &[String]) {
        match words.get(1).map(String::as_str) {
            Some("add") => {
                let Some(id) = words.get(2).and_then(|id| id.parse::<i32>().ok()) else {
                    self.status = "usage: room add <id>".to_string();
                    return;
                };
                if self.adventure.rooms.iter().any(|room| room.id == id) {
                    self.status = format!("room id already exists: {id}");
                    return;
                }
                self.adventure.rooms.push(RoomDef {
                    id,
                    name: format!("Room {}", id),
                    description: String::new(),
                    ..RoomDef::default()
                });
                self.selected_room = self.adventure.rooms.len() - 1;
                if self.adventure.start_room == 0 {
                    self.adventure.start_room = id;
                }
                self.dirty = true;
                self.status = format!("Added room {id}");
            }
            Some("del") => {
                let Some(id) = words
                    .get(2)
                    .and_then(|id| id.parse::<i32>().ok())
                    .or_else(|| self.selected_room().map(|room| room.id))
                else {
                    self.status = "no room selected".to_string();
                    return;
                };
                if self.adventure.rooms.len() <= 1 {
                    self.status = "An adventure needs at least one room".to_string();
                    return;
                }
                let Some(index) = self.adventure.rooms.iter().position(|room| room.id == id) else {
                    self.status = format!("no such room: {id}");
                    return;
                };
                self.adventure.rooms.remove(index);
                for room in &mut self.adventure.rooms {
                    room.exits.retain(|_, exit| exit.destination() != id);
                }
                for item in &mut self.adventure.items {
                    if item.location == id {
                        item.location = 0;
                    }
                }
                for monster in &mut self.adventure.monsters {
                    if monster.room_id == id {
                        monster.room_id = self
                            .adventure
                            .rooms
                            .first()
                            .map(|room| room.id)
                            .unwrap_or(0);
                    }
                }
                for shop in &mut self.adventure.shops {
                    if shop.room_id == id {
                        shop.room_id = self
                            .adventure
                            .rooms
                            .first()
                            .map(|room| room.id)
                            .unwrap_or(0);
                    }
                }
                if self.adventure.start_room == id {
                    self.adventure.start_room = self
                        .adventure
                        .rooms
                        .first()
                        .map(|room| room.id)
                        .unwrap_or(0);
                }
                self.clamp_selection();
                self.dirty = true;
                self.status = format!("Deleted room {id}");
            }
            Some("set") => {
                let Some(field) = words.get(2).map(String::as_str) else {
                    self.status = "usage: room set title|desc <text>".to_string();
                    return;
                };
                let Some(value) = words.get(3).cloned() else {
                    self.status = "usage: room set title|desc <text>".to_string();
                    return;
                };
                let Some(room) = self.selected_room_mut() else {
                    self.status = "no room selected".to_string();
                    return;
                };
                match field {
                    "title" | "name" => {
                        room.name = value;
                        self.dirty = true;
                        self.status = "Updated room title".to_string();
                    }
                    "desc" | "description" => {
                        room.description = value;
                        self.dirty = true;
                        self.status = "Updated room description".to_string();
                    }
                    _ => self.status = "usage: room set title|desc <text>".to_string(),
                }
            }
            _ => {
                self.status =
                    "usage: room add <id> | room del [id] | room set title|desc <text>".to_string();
            }
        }
    }

    fn exec_exit_command(&mut self, words: &[String]) {
        let Some(room) = self.selected_room_mut() else {
            self.status = "no room selected".to_string();
            return;
        };
        match words.get(1).map(String::as_str) {
            Some("set") => {
                let (Some(direction), Some(destination)) = (
                    words.get(2),
                    words.get(3).and_then(|id| id.parse::<i32>().ok()),
                ) else {
                    self.status = "usage: exit set <dir> <dest_room_id>".to_string();
                    return;
                };
                room.exits.insert(
                    direction.to_lowercase(),
                    mellivorex_rs::ExitDef::Plain(destination),
                );
                self.dirty = true;
                self.status = format!("Set exit '{direction}' -> {destination}");
            }
            Some("lock") => {
                let (Some(direction), Some(key)) = (words.get(2), words.get(3)) else {
                    self.status = "usage: exit lock <dir> <key>".to_string();
                    return;
                };
                let direction = direction.to_lowercase();
                match room.exits.get_mut(&direction) {
                    Some(exit) => {
                        exit.set_locked(true, Some(key.clone()));
                        self.dirty = true;
                        self.status = format!("Locked '{direction}' with '{key}'");
                    }
                    None => self.status = format!("No exit '{direction}'"),
                }
            }
            Some("unlock") => {
                let Some(direction) = words.get(2) else {
                    self.status = "usage: exit unlock <dir>".to_string();
                    return;
                };
                let direction = direction.to_lowercase();
                match room.exits.get_mut(&direction) {
                    Some(exit) => {
                        exit.set_locked(false, None);
                        self.dirty = true;
                        self.status = format!("Unlocked '{direction}'");
                    }
                    None => self.status = format!("No exit '{direction}'"),
                }
            }
            Some("del") => {
                let Some(direction) = words.get(2) else {
                    self.status = "usage: exit del <dir>".to_string();
                    return;
                };
                room.exits.remove(&direction.to_lowercase());
                self.dirty = true;
                self.status = format!("Deleted exit '{direction}'");
            }
            _ => self.status = "usage: exit set|lock|unlock|del ...".to_string(),
        }
    }

    fn exec_item_command(&mut self, words: &[String]) {
        match words.get(1).map(String::as_str) {
            Some("add") => {
                let (Some(id), Some(name)) = (
                    words.get(2).and_then(|id| id.parse::<i32>().ok()),
                    words.get(3),
                ) else {
                    self.status = "usage: item add <id> <name> <description>".to_string();
                    return;
                };
                if self.adventure.items.iter().any(|item| item.id == id) {
                    self.status = format!("item id already exists: {id}");
                    return;
                }
                let description = words.iter().skip(4).cloned().collect::<Vec<_>>().join(" ");
                let room_id = self.selected_room().map(|room| room.id).unwrap_or(0);
                self.adventure.items.push(mellivorex_rs::ItemDef {
                    id,
                    name: name.clone(),
                    description,
                    location: room_id,
                    ..mellivorex_rs::ItemDef::default()
                });
                if let Some(room) = self.selected_room_mut()
                    && !room.items.contains(&id)
                {
                    room.items.push(id);
                }
                self.dirty = true;
                self.status = format!("Added item '{name}'");
            }
            Some("del") => {
                let Some(key) = words.get(2) else {
                    self.status = "usage: item del <id|name>".to_string();
                    return;
                };
                let id = key.parse::<i32>().ok().or_else(|| {
                    self.adventure
                        .items
                        .iter()
                        .find(|item| item.name.eq_ignore_ascii_case(key))
                        .map(|item| item.id)
                });
                let Some(id) = id else {
                    self.status = format!("No item matches '{key}'");
                    return;
                };
                self.adventure.items.retain(|item| item.id != id);
                for room in &mut self.adventure.rooms {
                    room.items.retain(|item_id| *item_id != id);
                }
                for monster in &mut self.adventure.monsters {
                    if monster.weapon_id == Some(id) {
                        monster.weapon_id = None;
                    }
                    monster.loot.retain(|item_id| *item_id != id);
                }
                for shop in &mut self.adventure.shops {
                    shop.stock.retain(|stock| stock.item_id != id);
                }
                self.dirty = true;
                self.status = format!("Deleted item {id}");
            }
            Some("set") => {
                let (Some(id), Some(field), Some(value)) = (
                    words.get(2).and_then(|id| id.parse::<i32>().ok()),
                    words.get(3).map(String::as_str),
                    words.get(4),
                ) else {
                    self.status =
                        "usage: item set <id> name|desc|type|value|weight|location|light <value>"
                            .to_string();
                    return;
                };
                let Some(item) = self.adventure.items.iter_mut().find(|item| item.id == id) else {
                    self.status = format!("No item {id}");
                    return;
                };
                match field {
                    "name" => item.name = value.clone(),
                    "desc" | "description" => {
                        item.description =
                            words.iter().skip(4).cloned().collect::<Vec<_>>().join(" ");
                    }
                    "type" => {
                        item.item_type = match value.to_lowercase().as_str() {
                            "weapon" => ItemType::Weapon,
                            "armor" => ItemType::Armor,
                            "treasure" => ItemType::Treasure,
                            "readable" => ItemType::Readable,
                            "edible" => ItemType::Edible,
                            "drinkable" => ItemType::Drinkable,
                            "container" => ItemType::Container,
                            _ => ItemType::Normal,
                        };
                    }
                    "value" => item.value = value.parse().unwrap_or(item.value),
                    "weight" => item.weight = value.parse().unwrap_or(item.weight),
                    "location" => item.location = value.parse().unwrap_or(item.location),
                    "light" => item.is_light_source = matches!(value.as_str(), "on" | "true" | "1"),
                    _ => {
                        self.status = format!("Unknown item field '{field}'");
                        return;
                    }
                }
                self.dirty = true;
                self.status = format!("Updated item {id}");
            }
            Some("weapon") => {
                let (Some(id), Some(dice), Some(sides)) = (
                    words.get(2).and_then(|id| id.parse::<i32>().ok()),
                    words.get(3).and_then(|value| value.parse::<i32>().ok()),
                    words.get(4).and_then(|value| value.parse::<i32>().ok()),
                ) else {
                    self.status = "usage: item weapon <id> <dice> <sides>".to_string();
                    return;
                };
                if let Some(item) = self.adventure.items.iter_mut().find(|item| item.id == id) {
                    item.is_weapon = true;
                    item.item_type = ItemType::Weapon;
                    item.weapon_dice = dice;
                    item.weapon_sides = sides;
                    item.weapon_type = item.weapon_type.max(1);
                    self.dirty = true;
                    self.status = format!("Item {id} is now a weapon");
                } else {
                    self.status = format!("No item {id}");
                }
            }
            _ => self.status = "usage: item add|del|set|weapon ...".to_string(),
        }
    }

    fn exec_monster_command(&mut self, words: &[String]) {
        match words.get(1).map(String::as_str) {
            Some("add") => {
                let (Some(id), Some(room_id), Some(name)) = (
                    words.get(2).and_then(|id| id.parse::<i32>().ok()),
                    words.get(3).and_then(|id| id.parse::<i32>().ok()),
                    words.get(4),
                ) else {
                    self.status = "usage: monster add <id> <room_id> <name>".to_string();
                    return;
                };
                if self
                    .adventure
                    .monsters
                    .iter()
                    .any(|monster| monster.id == id)
                {
                    self.status = format!("monster id already exists: {id}");
                    return;
                }
                self.adventure.monsters.push(mellivorex_rs::MonsterDef {
                    id,
                    name: name.clone(),
                    room_id,
                    ..mellivorex_rs::MonsterDef::default()
                });
                self.dirty = true;
                self.status = format!("Added monster '{name}'");
            }
            Some("del") => {
                let Some(id) = words.get(2).and_then(|id| id.parse::<i32>().ok()) else {
                    self.status = "usage: monster del <id>".to_string();
                    return;
                };
                self.adventure.monsters.retain(|monster| monster.id != id);
                self.dirty = true;
                self.status = format!("Deleted monster {id}");
            }
            Some("set") => {
                let (Some(id), Some(field), Some(value)) = (
                    words.get(2).and_then(|id| id.parse::<i32>().ok()),
                    words.get(3).map(String::as_str),
                    words.get(4),
                ) else {
                    self.status =
                        "usage: monster set <id> name|desc|room|hp|agility|gold|status <value>"
                            .to_string();
                    return;
                };
                let Some(monster) = self.adventure.monsters.iter_mut().find(|m| m.id == id) else {
                    self.status = format!("No monster {id}");
                    return;
                };
                match field {
                    "name" => monster.name = value.clone(),
                    "desc" | "description" => {
                        monster.description =
                            words.iter().skip(4).cloned().collect::<Vec<_>>().join(" ");
                    }
                    "room" => monster.room_id = value.parse().unwrap_or(monster.room_id),
                    "hp" | "hardiness" => {
                        monster.hardiness = value.parse().unwrap_or(monster.hardiness)
                    }
                    "agility" => monster.agility = value.parse().unwrap_or(monster.agility),
                    "gold" => monster.gold = value.parse().unwrap_or(monster.gold),
                    "status" | "friendliness" => {
                        monster.friendliness = match value.to_lowercase().as_str() {
                            "friendly" => MonsterStatus::Friendly,
                            "hostile" => MonsterStatus::Hostile,
                            _ => MonsterStatus::Neutral,
                        }
                    }
                    _ => {
                        self.status = format!("Unknown monster field '{field}'");
                        return;
                    }
                }
                self.dirty = true;
                self.status = format!("Updated monster {id}");
            }
            _ => self.status = "usage: monster add|del|set ...".to_string(),
        }
    }

    fn exec_quest_command(&mut self, words: &[String]) {
        match words.get(1).map(String::as_str) {
            Some("add") => {
                let (Some(id), Some(title)) = (
                    words.get(2).and_then(|id| id.parse::<i32>().ok()),
                    words.get(3),
                ) else {
                    self.status = "usage: quest add <id> <title>".to_string();
                    return;
                };
                if self.adventure.quests.iter().any(|quest| quest.id == id) {
                    self.status = format!("quest id already exists: {id}");
                    return;
                }
                self.adventure.quests.push(QuestDef {
                    id,
                    title: title.clone(),
                    description: String::new(),
                    rewards: QuestRewardsDef {
                        gold: 50,
                        xp: 100,
                        ..QuestRewardsDef::default()
                    },
                    stages: vec![StageDef {
                        stage_id: "main".to_string(),
                        stage_number: 1,
                        title: "Main Objectives".to_string(),
                        ..StageDef::default()
                    }],
                    ..QuestDef::default()
                });
                self.dirty = true;
                self.status = format!("Added quest '{title}'");
            }
            Some("del") => {
                let Some(id) = words.get(2).and_then(|id| id.parse::<i32>().ok()) else {
                    self.status = "usage: quest del <id>".to_string();
                    return;
                };
                self.adventure.quests.retain(|quest| quest.id != id);
                self.dirty = true;
                self.status = format!("Deleted quest {id}");
            }
            Some("objective") => {
                let (Some(quest_id), Some(kind), Some(target)) = (
                    words.get(2).and_then(|id| id.parse::<i32>().ok()),
                    words.get(3).map(String::as_str),
                    words.get(4),
                ) else {
                    self.status =
                        "usage: quest objective <id> <kill|collect|reach|talk> <target> [count]"
                            .to_string();
                    return;
                };
                let count = words
                    .get(5)
                    .and_then(|value| value.parse::<i32>().ok())
                    .unwrap_or(1);
                let objective_kind = match kind {
                    "kill" => ObjectiveKind::KillMonster,
                    "collect" => ObjectiveKind::CollectItem,
                    "reach" => ObjectiveKind::ReachRoom,
                    "talk" => ObjectiveKind::TalkToNpc,
                    _ => {
                        self.status = "kind must be kill, collect, reach, or talk".to_string();
                        return;
                    }
                };
                let Some(quest) = self.adventure.quests.iter_mut().find(|q| q.id == quest_id)
                else {
                    self.status = format!("No quest {quest_id}");
                    return;
                };
                if quest.stages.is_empty() {
                    quest.stages.push(StageDef {
                        stage_id: "main".to_string(),
                        stage_number: 1,
                        title: "Main Objectives".to_string(),
                        ..StageDef::default()
                    });
                }
                let target_value = target
                    .parse::<i64>()
                    .map(|number| serde_json::Value::Number(number.into()))
                    .unwrap_or_else(|_| serde_json::Value::String(target.clone()));
                quest.stages[0]
                    .objectives
                    .push(ObjectiveDef::Structured(StructuredObjective {
                        kind: objective_kind,
                        target: Some(target.clone()),
                        target_id: Some(target_value),
                        description: format!("{} {}", kind, target),
                        count,
                        ..StructuredObjective::default()
                    }));
                self.dirty = true;
                self.status = format!("Added objective to quest {quest_id}");
            }
            _ => self.status = "usage: quest add|del|objective ...".to_string(),
        }
    }

    fn exec_shop_command(&mut self, words: &[String]) {
        match words.get(1).map(String::as_str) {
            Some("add") => {
                let (Some(id), Some(room_id), Some(name)) = (
                    words.get(2).and_then(|id| id.parse::<i32>().ok()),
                    words.get(3).and_then(|id| id.parse::<i32>().ok()),
                    words.get(4),
                ) else {
                    self.status = "usage: shop add <id> <room_id> <name>".to_string();
                    return;
                };
                self.adventure.shops.push(mellivorex_rs::ShopDef {
                    id,
                    name: name.clone(),
                    room_id,
                    gold: 200,
                    ..mellivorex_rs::ShopDef::default()
                });
                self.dirty = true;
                self.status = format!("Added shop '{name}'");
            }
            Some("del") => {
                let Some(id) = words.get(2).and_then(|id| id.parse::<i32>().ok()) else {
                    self.status = "usage: shop del <id>".to_string();
                    return;
                };
                self.adventure.shops.retain(|shop| shop.id != id);
                self.dirty = true;
                self.status = format!("Deleted shop {id}");
            }
            Some("stock") => {
                let (Some(shop_id), Some(item_id)) = (
                    words.get(2).and_then(|id| id.parse::<i32>().ok()),
                    words.get(3).and_then(|id| id.parse::<i32>().ok()),
                ) else {
                    self.status = "usage: shop stock <shop_id> <item_id> [qty]".to_string();
                    return;
                };
                let quantity = words
                    .get(4)
                    .and_then(|value| value.parse::<i32>().ok())
                    .unwrap_or(1);
                let Some(shop) = self.adventure.shops.iter_mut().find(|s| s.id == shop_id) else {
                    self.status = format!("No shop {shop_id}");
                    return;
                };
                if let Some(stock) = shop.stock.iter_mut().find(|stock| stock.item_id == item_id) {
                    stock.quantity += quantity;
                } else {
                    shop.stock.push(mellivorex_rs::ShopStockDef {
                        item_id,
                        quantity,
                        ..mellivorex_rs::ShopStockDef::default()
                    });
                }
                self.dirty = true;
                self.status = format!("Shop {shop_id} stocks item {item_id}");
            }
            _ => self.status = "usage: shop add|del|stock ...".to_string(),
        }
    }
}

fn parse_words(input: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote: Option<char> = None;

    while let Some(character) = chars.next() {
        match quote {
            Some(active) => {
                if character == active {
                    quote = None;
                } else if character == '\\' {
                    if let Some(next) = chars.next() {
                        current.push(next);
                    }
                } else {
                    current.push(character);
                }
            }
            None => {
                if character.is_whitespace() {
                    if !current.is_empty() {
                        out.push(std::mem::take(&mut current));
                    }
                } else if character == '"' || character == '\'' {
                    quote = Some(character);
                } else {
                    current.push(character);
                }
            }
        }
    }

    if quote.is_some() {
        return Err("unterminated quote".to_string());
    }
    if !current.is_empty() {
        out.push(current);
    }
    Ok(out)
}

fn run(tui: &mut Tui, app: &mut App) -> anyhow::Result<()> {
    loop {
        app.clamp_selection();
        tui.terminal.draw(|frame| {
            let size = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
                .split(size);
            draw_main(frame, chunks[0], app);
            draw_status(frame, chunks[1], app);
        })?;

        if app.force_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && handle_key(app, key)
        {
            break;
        }
    }
    Ok(())
}

fn draw_main(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)].as_ref())
        .split(area);

    let items: Vec<ListItem> = app
        .adventure
        .rooms
        .iter()
        .enumerate()
        .map(|(index, room)| {
            let marker = if index == app.selected_room { ">" } else { " " };
            ListItem::new(format!("{} {}: {}", marker, room.id, room.name))
        })
        .collect();
    let list = List::new(items).block(Block::default().title(" Rooms ").borders(Borders::ALL));
    frame.render_widget(list, columns[0]);

    let details = Paragraph::new(room_details_text(app))
        .block(Block::default().title(" Details ").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(details, columns[1]);
}

fn room_details_text(app: &App) -> Text<'static> {
    let Some(room) = app.selected_room() else {
        return Text::from("No room selected. Use ':room add <id>' to create one.");
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Room ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                room.id.to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — "),
            Span::styled(
                room.name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(room.description.clone()),
    ];
    if room.is_dark {
        lines.push(Line::from(Span::styled(
            "Dark room (requires a light source).",
            Style::default().add_modifier(Modifier::ITALIC),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Exits:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    if room.exits.is_empty() {
        lines.push(Line::from("  (none)"));
    }
    for (direction, exit) in &room.exits {
        let lock = exit
            .lock()
            .filter(|lock| lock.locked)
            .map(|lock| {
                format!(
                    " [locked: {}]",
                    lock.key.clone().unwrap_or_else(|| "no key".to_string())
                )
            })
            .unwrap_or_default();
        lines.push(Line::from(format!(
            "  {} -> {}{}",
            direction,
            exit.destination(),
            lock
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Items:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    if room.items.is_empty() {
        lines.push(Line::from("  (none)"));
    }
    for item_id in &room.items {
        if let Some(item) = app.adventure.items.iter().find(|item| item.id == *item_id) {
            let flags = [
                item.is_weapon.then_some("weapon"),
                item.is_armor.then_some("armor"),
                item.is_light_source.then_some("light"),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(", ");
            lines.push(Line::from(format!(
                "  [{}] {}{}",
                item.id,
                item.name,
                if flags.is_empty() {
                    String::new()
                } else {
                    format!(" ({flags})")
                }
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Monsters:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    let monsters: Vec<_> = app
        .adventure
        .monsters
        .iter()
        .filter(|monster| monster.room_id == room.id)
        .collect();
    if monsters.is_empty() {
        lines.push(Line::from("  (none)"));
    }
    for monster in monsters {
        lines.push(Line::from(format!(
            "  [{}] {} ({:?}) hp {}",
            monster.id, monster.name, monster.friendliness, monster.hardiness
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Quests:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    if app.adventure.quests.is_empty() {
        lines.push(Line::from("  (none)"));
    }
    for quest in &app.adventure.quests {
        let objectives: usize = quest
            .stages
            .iter()
            .map(|stage| stage.objectives.len())
            .sum();
        lines.push(Line::from(format!(
            "  [{}] {} ({} objective(s))",
            quest.id, quest.title, objectives
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Shops:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    let shops: Vec<_> = app
        .adventure
        .shops
        .iter()
        .filter(|shop| shop.room_id == room.id)
        .collect();
    if shops.is_empty() {
        lines.push(Line::from("  (none)"));
    }
    for shop in shops {
        lines.push(Line::from(format!(
            "  [{}] {} ({} gold, {} stock line(s))",
            shop.id,
            shop.name,
            shop.gold,
            shop.stock.len()
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "Adventure: {} | start room {} | {} rooms, {} items, {} monsters, {} quests",
            app.adventure.title,
            app.adventure.start_room,
            app.adventure.rooms.len(),
            app.adventure.items.len(),
            app.adventure.monsters.len(),
            app.adventure.quests.len()
        ),
        Style::default().add_modifier(Modifier::DIM),
    )));

    Text::from(lines)
}

fn draw_status(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let (text, style) = match app.mode {
        Mode::Command => (
            format!(":{}", app.cmd),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Mode::Normal => (
            format!(
                "{}  [{}]{}",
                app.status,
                app.file.display(),
                if app.dirty { " *unsaved*" } else { "" }
            ),
            Style::default(),
        ),
    };
    let paragraph = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match app.mode {
        Mode::Normal => handle_key_normal(app, key),
        Mode::Command => handle_key_command(app, key),
    }
}

fn handle_key_normal(app: &mut App, key: KeyEvent) -> bool {
    match (key.code, key.modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => true,
        (KeyCode::Char('q'), _) => {
            if app.dirty {
                if app.quit_confirm {
                    true
                } else {
                    app.quit_confirm = true;
                    app.status =
                        "Unsaved changes. Press q again to quit, or save with s.".to_string();
                    false
                }
            } else {
                true
            }
        }
        (KeyCode::Char('s'), _) => {
            app.save();
            false
        }
        (KeyCode::Char(':'), _) => {
            app.mode = Mode::Command;
            app.cmd.clear();
            app.quit_confirm = false;
            false
        }
        (KeyCode::Up, _) => {
            if app.selected_room > 0 {
                app.selected_room -= 1;
            }
            app.quit_confirm = false;
            false
        }
        (KeyCode::Down, _) => {
            if app.selected_room + 1 < app.adventure.rooms.len() {
                app.selected_room += 1;
            }
            app.quit_confirm = false;
            false
        }
        _ => false,
    }
}

fn handle_key_command(app: &mut App, key: KeyEvent) -> bool {
    match (key.code, key.modifiers) {
        (KeyCode::Esc, _) => {
            app.mode = Mode::Normal;
            app.cmd.clear();
            false
        }
        (KeyCode::Enter, _) => {
            let command = app.cmd.clone();
            app.cmd.clear();
            app.mode = Mode::Normal;
            app.exec_command(&command);
            app.force_quit
        }
        (KeyCode::Backspace, _) => {
            app.cmd.pop();
            false
        }
        (KeyCode::Char(character), KeyModifiers::NONE)
        | (KeyCode::Char(character), KeyModifiers::SHIFT) => {
            app.cmd.push(character);
            false
        }
        _ => false,
    }
}
