//! A tiny command runner built on top of [`DB`](crate::db::DB).

use crate::prelude::*;
use std::error::Error;
use std::sync::Mutex;

/// Stateful command runner for `SET`, `ADD`, and `GET`.
pub struct Scuver {
    db: Mutex<DB>,
}

impl Scuver {
    /// Creates a new runner with a default seeded database.
    pub fn new() -> Self {
        Self {
            db: Mutex::new(DB::new("default", String::new())),
        }
    }

    /// Executes a single command string.
    ///
    /// Supported commands are `SET <key> <value>`, `ADD <key> <value>`, and
    /// `GET <key>`.
    pub fn run(&self, code: String) -> Result<String, Box<dyn Error>> {
        let command: Vec<&str> = code.trim().split_whitespace().collect();

        if command.is_empty() {
            return Err("The code is empty".into());
        }

        match command[0] {
            "SET" => {
                if ensure_arg(ArgType::Set, command.len()) {
                    let mut db = self.db.lock().unwrap();
                    db.set(command[1], command[2].to_string());
                }
            }
            "ADD" => {
                if ensure_arg(ArgType::Add, command.len()) {
                    let mut db = self.db.lock().unwrap();
                    db.add_row(command[1], command[2].to_string());
                }
            }
            "GET" => {
                if ensure_arg(ArgType::Get, command.len()) {
                    let db = self.db.lock().unwrap();
                    let values = db.get_display(command[1]);
                    return Ok(values.join("\n"));
                }
            }
            _ => {
                eprintln!("kelpdb: command not found");
            }
        }
        Ok(String::new())
    }
}

enum ArgType {
    Set,
    Add,
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
        ArgType::Add => {
            if command_len != 3 {
                eprintln!("kelpdb: ADD requires 2 arguments (key value)");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_set() {
        let scuver = Scuver::new();
        let result = scuver.run("SET key value".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_get() {
        let scuver = Scuver::new();
        scuver.run("SET key value".to_string()).unwrap();
        let result = scuver.run("GET key".to_string()).unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn test_run_add() {
        let scuver = Scuver::new();
        let result = scuver.run("ADD new_key initial_value".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_empty_code() {
        let scuver = Scuver::new();
        let result = scuver.run("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_run_invalid_command() {
        let scuver = Scuver::new();
        let result = scuver.run("INVALID".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_get_after_multiple_adds() {
        let scuver = Scuver::new();

        scuver.run("ADD posts first".to_string()).unwrap();
        scuver.run("ADD posts second".to_string()).unwrap();

        let result = scuver.run("GET posts".to_string()).unwrap();
        assert_eq!(result, "first\nsecond");
    }

    #[test]
    fn test_run_get_missing_key_returns_empty_string() {
        let scuver = Scuver::new();

        let result = scuver.run("GET missing".to_string()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_run_ignores_extra_whitespace() {
        let scuver = Scuver::new();

        scuver.run("   SET   key   value   ".to_string()).unwrap();
        let result = scuver.run(" GET key ".to_string()).unwrap();

        assert_eq!(result, "value");
    }

    #[test]
    fn test_run_invalid_arity_returns_empty_string() {
        let scuver = Scuver::new();

        let result = scuver.run("SET only_key".to_string()).unwrap();
        assert_eq!(result, "");
    }
}
