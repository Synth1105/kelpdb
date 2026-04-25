use kelpdb::prelude::DB;
use kelpdb::scuver::Scuver;
use rusty_repl::*;
use std::error::Error;
use std::sync::{Mutex, OnceLock};

static GLOBAL_RUNNER: OnceLock<Mutex<Scuver>> = OnceLock::new();

fn input_handler(cmd: String) -> bool {
    match cmd.trim() {
        ":exit" | ":quit" => true,
        val => {
            let runner = GLOBAL_RUNNER.get_or_init(|| {
                let db = DB::new("__example__", String::from("Hello, KelpDB"));
                Mutex::new(Scuver::new(db))
            });
            let runner = runner.lock().unwrap();

            match runner.run(val.to_string()) {
                Ok(output) => {
                    if !output.is_empty() {
                        println!("{output}");
                    }
                }
                Err(err) => eprintln!("kelpdb: {err}"),
            }

            false
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("KelpDB REPL");
    let ks = KeywordStyle::new(vec!["GET", "SET", "RM", ":exit", ":quit"], Color::Cyan);

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
