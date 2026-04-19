use std::io::{self, Write};

use sagacraft_rs::Engine;

const DEFAULT_ADVENTURE: &str = "examples/adventures/shattered_realms_demo.json";

fn main() {
    let adventure_path = parse_args(std::env::args().skip(1));

    let mut engine = match Engine::load(&adventure_path) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Failed to load adventure '{}': {}", adventure_path, err);
            std::process::exit(1);
        }
    };

    println!("SagaCraft — CLI Player");
    println!("Type 'help' for commands. Type 'quit' to exit.\n");

    // Print intro/banner text from adventure file, then room description
    let intro = engine.intro();
    if !intro.is_empty() {
        println!("{}\n", intro);
    }
    println!("{}", engine.look());

    let stdin = io::stdin();
    loop {
        if engine.is_over() {
            println!("\n--- Game Over ---");
            break;
        }

        print!("> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            println!("Failed to read input.");
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "quit" | "q" | "exit" => break,
            _ => {
                for line in engine.send(input) {
                    println!("{}", line);
                }
            }
        }
    }
}

fn parse_args(mut args: impl Iterator<Item = String>) -> String {
    let mut adventure_path: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage_and_exit();
            }
            "--adventure" | "-a" => {
                if let Some(path) = args.next() {
                    adventure_path = Some(path);
                } else {
                    eprintln!("--adventure requires a path argument.");
                    print_usage_and_exit();
                }
            }
            other if !other.starts_with('-') => {
                // Support positional argument: sagacraft_player my_adventure.json
                adventure_path = Some(other.to_string());
            }
            unknown => {
                eprintln!("Unknown flag: {}", unknown);
                print_usage_and_exit();
            }
        }
    }

    adventure_path.unwrap_or_else(|| DEFAULT_ADVENTURE.to_string())
}

fn print_usage_and_exit() -> ! {
    println!("SagaCraft — CLI Player");
    println!("Usage:");
    println!("  sagacraft_player [<adventure.json>]");
    println!("  sagacraft_player --adventure <path>");
    println!();
    println!("Options:");
    println!(
        "  -a, --adventure <path>    Adventure JSON file to load (default: {})",
        DEFAULT_ADVENTURE
    );
    println!("  -h, --help                Show this help");
    std::process::exit(0)
}
