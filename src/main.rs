use kelpdb::prelude::*;
use rusty_repl::*;
use std::error::Error;
use std::sync::{Mutex, OnceLock};

static GLOBAL_DB: OnceLock<Mutex<DB>> = OnceLock::new();

enum ArgType {
    Set,

    Get,
}

fn ensure_arg(argtype: ArgType, command_len: usize) -> bool {
    match argtype {
        ArgType::Set => {
            if command_len != 3 {
                eprintln!("kelpdb: SET requires 2 arguments (key value)");
                return false;
            }
        }

        ArgType::Get => {
            if command_len != 2 {
                eprintln!("kelpdb: GET requires 1 argument (key)");
                return false;
            }
        }
    }
    true
}

fn process(orig_command: String, db_mutex: &Mutex<DB>) {
    let command: Vec<&str> = orig_command.trim().split_whitespace().collect();

    if command.is_empty() {
        return;
    }

    match command[0] {
        "SET" => {
            if ensure_arg(ArgType::Set, command.len()) {
                let mut db = db_mutex.lock().unwrap();
                db.set(command[1], command[2].to_string());
            }
        }

        "GET" => {
            if ensure_arg(ArgType::Get, command.len()) {
                let db = db_mutex.lock().unwrap();
                let result = db.get_display(command[1]);
                println!("{:#?}", result);
            }
        }
        _ => {
            eprintln!("kelpdb: command not found");
        }
    }
}

fn input_handler(cmd: String) -> bool {
    let db_mutex =
        GLOBAL_DB.get_or_init(|| Mutex::new(DB::new("example", String::from("Hello, World!"))));

    match cmd.trim() {
        ":exit" | ":quit" => return true,
        val => process(val.to_string(), db_mutex),
    }

    false
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("KelpDB REPL");
    let ks = KeywordStyle::new(vec!["GET", "SET", ":exit", ":quit"], Color::Cyan);

    let default_prompt = CleanPrompt::from(
        DefaultPromptSegment::Basic("KelpDB ❯ ".to_string()),
        DefaultPromptSegment::CurrentDateTime,
    );

    let cfg = ReplConfig::new("KELPDB")
        .with_kw_style(ks)
        .with_prompt(default_prompt);

    let repl_manager = Repl::from(cfg);
    let _ = repl_manager.run(input_handler);
    Ok(())
}
