use std::io;
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

use sagacraft_rs::{Adventure, AdventureItem, AdventureRoom};

fn main() -> anyhow::Result<()> {
    let args = Args::parse(std::env::args().skip(1).collect());
    if args.show_help {
        print_help();
        return Ok(());
    }

    let is_new = args.new.is_some();

    let file_path = if let Some(p) = &args.file {
        p.clone()
    } else if let Some(p) = &args.new {
        p.clone()
    } else {
        PathBuf::from("demo_adventure.json")
    };

    let mut app = if is_new {
        App::new_with_file(file_path, Adventure::demo())
    } else {
        match Adventure::load_json_file(&file_path) {
            Ok(adv) => App::new_with_file(file_path, adv),
            Err(_) => App::new_with_file(file_path, Adventure::demo()),
        }
    };

    let mut tui = Tui::new()?;
    let res = run(&mut tui, &mut app);
    tui.shutdown()?;

    res
}

fn print_help() {
    println!("SagaCraft IDE (Rust TUI)");
    println!("Usage:");
    println!("  sagacraft_ide_tui --file <path>");
    println!("  sagacraft_ide_tui --new <path>");
    println!();
    println!("Keys:");
    println!("  Up/Down   select room");
    println!("  :         command mode");
    println!("  Esc       normal mode");
    println!("  q         quit");
    println!("  s         save");
    println!();
    println!("Commands (type after ':' then Enter):");
    println!("  w | write              save");
    println!("  q | quit               quit");
    println!("  wq                     save then quit");
    println!("  help                   show help");
    println!("  set start <room_id>    set start room");
    println!("  room add <id>          add room");
    println!("  room del <id?>         delete room (default selected)");
    println!("  room set title <text>  set selected room title");
    println!("  room set desc <text>   set selected room description");
    println!("  exit set <dir> <dest>  set selected room exit");
    println!("  exit del <dir>         delete selected room exit");
    println!("  item add <id> <name> <desc>   add item to selected room");
    println!("  item del <name>               delete item by name");
    println!();
    println!("Tip: quote values with spaces, e.g. item add key \"Ancient Key\" \"A key.\"");
}

#[derive(Debug, Default)]
struct Args {
    file: Option<PathBuf>,
    new: Option<PathBuf>,
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
                "--file" | "-f" => {
                    if let Some(p) = argv.first().cloned() {
                        argv.remove(0);
                        out.file = Some(PathBuf::from(p));
                    } else {
                        out.show_help = true;
                        break;
                    }
                }
                "--new" => {
                    if let Some(p) = argv.first().cloned() {
                        argv.remove(0);
                        out.new = Some(PathBuf::from(p));
                    } else {
                        out.show_help = true;
                        break;
                    }
                }
                _ => {
                    // Treat a single positional arg as --file <path>.
                    if out.file.is_none() && out.new.is_none() {
                        out.file = Some(PathBuf::from(arg));
                    }
                }
            }
        }
        out
    }
}

struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
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
    adventure: Adventure,
    selected_room: usize,
    mode: Mode,
    cmd: String,
    status: String,
    dirty: bool,
    quit_confirm: bool,
}

impl App {
    fn new_with_file(file: PathBuf, adventure: Adventure) -> Self {
        let selected_room = 0;
        Self {
            file,
            adventure,
            selected_room,
            mode: Mode::Normal,
            cmd: String::new(),
            status: "Press ':' for commands. 's' to save.".to_string(),
            dirty: false,
            quit_confirm: false,
        }
    }

    fn selected_room_mut(&mut self) -> Option<&mut AdventureRoom> {
        self.adventure.rooms.get_mut(self.selected_room)
    }

    fn selected_room(&self) -> Option<&AdventureRoom> {
        self.adventure.rooms.get(self.selected_room)
    }

    fn clamp_selection(&mut self) {
        if self.adventure.rooms.is_empty() {
            self.selected_room = 0;
        } else if self.selected_room >= self.adventure.rooms.len() {
            self.selected_room = self.adventure.rooms.len() - 1;
        }
    }

    fn save(&mut self) {
        match self.adventure.save_json_file(&self.file) {
            Ok(()) => {
                self.status = format!("Saved {}", self.file.display());
                self.dirty = false;
                self.quit_confirm = false;
            }
            Err(e) => {
                self.status = format!("Save failed: {e}");
            }
        }
    }

    fn exec_command(&mut self, raw: &str) {
        let line = raw.trim();
        if line.is_empty() {
            return;
        }

        let words = match parse_words(line) {
            Ok(w) => w,
            Err(msg) => {
                self.status = format!("Command parse error: {msg}");
                return;
            }
        };

        let Some(cmd0) = words.first().map(|s| s.as_str()) else {
            return;
        };

        match cmd0 {
            "help" => {
                self.status = "Commands: w, q, wq, set start <room>, room add/del/set, exit set/del, item add/del".to_string();
            }
            "w" | "write" => self.save(),
            "q" | "quit" => {
                self.status = "quit".to_string();
            }
            "wq" => {
                self.save();
                self.status = "quit".to_string();
            }
            "set" => {
                if words.get(1).map(|s| s.as_str()) == Some("start") {
                    if let Some(room_id) = words.get(2) {
                        self.adventure.start_room = room_id.clone();
                        self.dirty = true;
                        self.status = format!("start_room set to '{}'", room_id);
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
            _ => {
                self.status = format!("Unknown command: {cmd0}");
            }
        }
    }

    fn exec_room_command(&mut self, words: &[String]) {
        match words.get(1).map(|s| s.as_str()) {
            Some("add") => {
                let Some(id) = words.get(2) else {
                    self.status = "usage: room add <id>".to_string();
                    return;
                };
                if self.adventure.rooms.iter().any(|r| r.id == *id) {
                    self.status = format!("room id already exists: {id}");
                    return;
                }
                self.adventure.rooms.push(AdventureRoom {
                    id: id.clone(),
                    title: id.clone(),
                    description: "".to_string(),
                    exits: Default::default(),
                    items: vec![],
                });
                self.selected_room = self.adventure.rooms.len() - 1;
                if self.adventure.start_room.trim().is_empty() {
                    self.adventure.start_room = id.clone();
                }
                self.dirty = true;
                self.status = format!("Added room '{id}'");
            }
            Some("del") => {
                let id = words
                    .get(2)
                    .cloned()
                    .or_else(|| self.selected_room().map(|r| r.id.clone()));
                let Some(id) = id else {
                    self.status = "no room selected".to_string();
                    return;
                };
                let Some(idx) = self.adventure.rooms.iter().position(|r| r.id == id) else {
                    self.status = format!("no such room: {id}");
                    return;
                };
                self.adventure.rooms.remove(idx);
                self.clamp_selection();
                self.dirty = true;
                self.status = format!("Deleted room '{id}'");
            }
            Some("set") => {
                let Some(field) = words.get(2).map(|s| s.as_str()) else {
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
                    "title" => {
                        room.title = value;
                        self.dirty = true;
                        self.status = "Updated room title".to_string();
                    }
                    "desc" | "description" => {
                        room.description = value;
                        self.dirty = true;
                        self.status = "Updated room description".to_string();
                    }
                    _ => {
                        self.status = "usage: room set title|desc <text>".to_string();
                    }
                }
            }
            _ => {
                self.status = "usage: room add <id> | room del <id?> | room set title|desc <text>"
                    .to_string();
            }
        }
    }

    fn exec_exit_command(&mut self, words: &[String]) {
        let Some(room) = self.selected_room_mut() else {
            self.status = "no room selected".to_string();
            return;
        };

        match words.get(1).map(|s| s.as_str()) {
            Some("set") => {
                let (Some(dir), Some(dest)) = (words.get(2), words.get(3)) else {
                    self.status = "usage: exit set <dir> <dest_room_id>".to_string();
                    return;
                };
                room.exits.insert(dir.clone(), dest.clone());
                self.dirty = true;
                self.status = format!("Set exit '{}' -> '{}'", dir, dest);
            }
            Some("del") => {
                let Some(dir) = words.get(2) else {
                    self.status = "usage: exit del <dir>".to_string();
                    return;
                };
                room.exits.remove(dir);
                self.dirty = true;
                self.status = format!("Deleted exit '{dir}'");
            }
            _ => {
                self.status = "usage: exit set <dir> <dest> | exit del <dir>".to_string();
            }
        }
    }

    fn exec_item_command(&mut self, words: &[String]) {
        let Some(room) = self.selected_room_mut() else {
            self.status = "no room selected".to_string();
            return;
        };

        match words.get(1).map(|s| s.as_str()) {
            Some("add") => {
                let (Some(id), Some(name), Some(desc)) = (words.get(2), words.get(3), words.get(4))
                else {
                    self.status = "usage: item add <id> <name> <desc>".to_string();
                    return;
                };
                room.items.push(AdventureItem {
                    id: id.clone(),
                    name: name.clone(),
                    description: desc.clone(),
                });
                self.dirty = true;
                self.status = format!("Added item '{name}'");
            }
            Some("del") => {
                let Some(name) = words.get(2) else {
                    self.status = "usage: item del <name>".to_string();
                    return;
                };
                let idx = room
                    .items
                    .iter()
                    .position(|i| i.name.eq_ignore_ascii_case(name));
                if let Some(i) = idx {
                    let removed = room.items.remove(i);
                    self.dirty = true;
                    self.status = format!("Deleted item '{}'", removed.name);
                } else {
                    self.status = "No such item in room".to_string();
                }
            }
            _ => {
                self.status = "usage: item add <id> <name> <desc> | item del <name>".to_string();
            }
        }
    }
}

fn parse_words(s: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars().peekable();
    let mut quote: Option<char> = None;

    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else if c == '\\' {
                    if let Some(n) = chars.next() {
                        cur.push(n);
                    }
                } else {
                    cur.push(c);
                }
            }
            None => {
                if c.is_whitespace() {
                    if !cur.is_empty() {
                        out.push(cur.clone());
                        cur.clear();
                    }
                } else if c == '"' || c == '\'' {
                    quote = Some(c);
                } else {
                    cur.push(c);
                }
            }
        }
    }

    if quote.is_some() {
        return Err("unterminated quote".to_string());
    }

    if !cur.is_empty() {
        out.push(cur);
    }

    Ok(out)
}

fn run(tui: &mut Tui, app: &mut App) -> anyhow::Result<()> {
    loop {
        app.clamp_selection();

        tui.terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
                .split(size);

            draw_main(f, chunks[0], app);
            draw_status(f, chunks[1], app);
        })?;

        if app.status == "quit" {
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

fn draw_main(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
        .split(area);

    let items: Vec<ListItem> = app
        .adventure
        .rooms
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mut style = Style::default();
            if i == app.selected_room {
                style = style.add_modifier(Modifier::BOLD);
            }
            let start_mark = if r.id == app.adventure.start_room {
                "*"
            } else {
                " "
            };
            ListItem::new(Line::from(vec![
                Span::raw(start_mark),
                Span::raw(" "),
                Span::styled(r.id.clone(), style),
            ]))
        })
        .collect();

    let rooms = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Rooms ({})", app.adventure.rooms.len())),
    );
    f.render_widget(rooms, columns[0]);

    let details = room_details_text(app);
    let detail_widget = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: false });
    f.render_widget(detail_widget, columns[1]);
}

fn room_details_text(app: &App) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(Line::from(format!(
        "Adventure: {} ({})",
        app.adventure.title, app.adventure.id
    )));
    lines.push(Line::from(format!(
        "Start room: {}",
        app.adventure.start_room
    )));
    lines.push(Line::from(""));

    let Some(room) = app.selected_room() else {
        lines.push(Line::from("No rooms."));
        return Text::from(lines);
    };

    lines.push(Line::from(format!("Room: {}", room.id)));
    lines.push(Line::from(format!("Title: {}", room.title)));
    lines.push(Line::from("Description:"));
    if room.description.trim().is_empty() {
        lines.push(Line::from("  (empty)"));
    } else {
        for l in room.description.lines() {
            lines.push(Line::from(format!("  {l}")));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from("Exits:"));
    if room.exits.is_empty() {
        lines.push(Line::from("  (none)"));
    } else {
        let mut exits: Vec<_> = room.exits.iter().collect();
        exits.sort_by(|a, b| a.0.cmp(b.0));
        for (dir, dest) in exits {
            lines.push(Line::from(format!("  {dir} -> {dest}")));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from("Items:"));
    if room.items.is_empty() {
        lines.push(Line::from("  (none)"));
    } else {
        for it in &room.items {
            lines.push(Line::from(format!("  {}: {}", it.id, it.name)));
        }
    }

    Text::from(lines)
}

fn draw_status(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let left_right = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(area);

    let dirty = if app.dirty { "*" } else { "" };
    let status = Paragraph::new(app.status.clone()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Status{dirty}")),
    );
    f.render_widget(status, left_right[0]);

    let mode = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Command => "COMMAND",
    };
    let cmd_line = match app.mode {
        Mode::Normal => format!("{mode}  file: {}", app.file.display()),
        Mode::Command => format!(":{}", app.cmd),
    };

    let cmd = Paragraph::new(cmd_line).block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(cmd, left_right[1]);
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
            let cmd = app.cmd.clone();
            app.cmd.clear();
            app.mode = Mode::Normal;
            app.exec_command(&cmd);
            // If command requested quit (via status), exit.
            app.status == "quit"
        }
        (KeyCode::Backspace, _) => {
            app.cmd.pop();
            false
        }
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            app.cmd.push(c);
            false
        }
        _ => false,
    }
}
