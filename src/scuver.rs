//! A tiny command runner built on top of [`DB`](crate::db::DB).

use crate::db::DB;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

/// Stateful command runner for `SET` and `GET`.
pub struct Scuver {
    db: Mutex<DB>,
}

impl Scuver {
    /// Creates a new runner from an existing database handle.
    pub fn new(db: DB) -> Self {
        Self { db: Mutex::new(db) }
    }

    /// Executes scuver code from file.
    pub fn load(file: &str) -> Result<String, Box<dyn Error>> {
        if Path::new(file).extension().and_then(|ext| ext.to_str()) != Some("scv") {
            Err("file ext should be scv")?
        }

        let filecontent = fs::read_to_string(file)?;
        let runner = Self::new(DB::new("__main__", true));
        let mut result = String::new();

        for line in filecontent
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            result.push_str(&runner.run(line.to_string())?);
        }

        Ok(result)
    }

    /// Executes a single command string.
    ///
    /// `SET` accepts typed values:
    /// - quoted values like `SET user "John"` are stored as `String`
    /// - bare integers like `SET age 1` are stored as `i64`
    /// - bare floats like `SET height 180.5` are stored as `f64`
    /// - bare booleans like `SET active true` are stored as `bool`
    pub fn run(&self, code: String) -> Result<String, Box<dyn Error>> {
        match parse_command(code.trim())? {
            Command::Set { key, value } => {
                let mut db = self.db.lock().unwrap();
                apply_value(&mut db, key, value);
                Ok(String::new())
            }
            Command::Get { key } => {
                let db = self.db.lock().unwrap();
                Ok(db.get_display(key).join("\n"))
            }
            Command::Rm { key } => {
                let mut db = self.db.lock().unwrap();
                db.remove(key);
                Ok(String::new())
            }
        }
    }
}

enum Command {
    Set { key: String, value: ParsedValue },
    Get { key: String },
    Rm  { key: String },
}

enum ParsedValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

fn apply_value(db: &mut DB, key: String, value: ParsedValue) {
    match value {
        ParsedValue::String(value) => db.set(key, value),
        ParsedValue::Integer(value) => db.set(key, value),
        ParsedValue::Float(value) => db.set(key, value),
        ParsedValue::Boolean(value) => db.set(key, value),
    }
}

fn parse_command(input: &str) -> Result<Command, Box<dyn Error>> {
    let (command, rest) = take_token(input).ok_or("The code is empty")?;

    match command {
        "SET" => {
            let (key, rest) = take_token(rest).ok_or("SET requires 2 arguments (key value)")?;
            let value = parse_value(rest.trim())?;
            Ok(Command::Set {
                key: key.to_string(),
                value,
            })
        }
        "GET" => {
            let (key, rest) = take_token(rest).ok_or("GET requires 1 argument (key)")?;
            if !rest.trim().is_empty() {
                return Err("GET requires 1 argument (key)".into());
            }
            Ok(Command::Get {
                key: key.to_string(),
            })
        }
        "RM" => {
            let (key, rest) = take_token(rest).ok_or("RM requires 1 arguments (key)")?;
            if !rest.trim().is_empty() {
                return Err("RM requires 1 argument (key)".into());
            }
            Ok(Command::Rm {
                key: key.to_string(),
            }) 

        }
        _ => Err("command not found".into()),
    }
}

fn parse_value(input: &str) -> Result<ParsedValue, Box<dyn Error>> {
    if input.is_empty() {
        return Err("SET requires 2 arguments (key value)".into());
    }

    if input.starts_with('"') {
        return parse_quoted_string(input).map(ParsedValue::String);
    }

    if input == "true" {
        return Ok(ParsedValue::Boolean(true));
    }

    if input == "false" {
        return Ok(ParsedValue::Boolean(false));
    }

    if let Ok(value) = input.parse::<i64>() {
        return Ok(ParsedValue::Integer(value));
    }

    if let Ok(value) = input.parse::<f64>() {
        return Ok(ParsedValue::Float(value));
    }

    Err("string values must be wrapped in double quotes".into())
}

fn parse_quoted_string(input: &str) -> Result<String, Box<dyn Error>> {
    let mut escaped = false;
    let mut parsed = String::new();
    let mut chars = input.chars();

    if chars.next() != Some('"') {
        return Err("string values must be wrapped in double quotes".into());
    }

    while let Some(ch) = chars.next() {
        if escaped {
            let resolved = match ch {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            };
            parsed.push(resolved);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => {
                if chars.as_str().trim().is_empty() {
                    return Ok(parsed);
                }
                return Err("unexpected trailing characters after string value".into());
            }
            other => parsed.push(other),
        }
    }

    Err("unterminated string literal".into())
}

fn take_token(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let split_at = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
    Some((&trimmed[..split_at], &trimmed[split_at..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_scuver() -> Scuver {
        Scuver::new(DB::new("default", String::new()))
    }

    #[test]
    fn stores_quoted_strings_as_strings() {
        let scuver = new_scuver();

        scuver.run(r#"SET key "value""#.to_string()).unwrap();
        let result = scuver.run("GET key".to_string()).unwrap();

        assert_eq!(result, "value");
    }

    #[test]
    fn stores_bare_integers_as_integers() {
        let scuver = new_scuver();

        scuver.run("SET key 1".to_string()).unwrap();
        let result = scuver.run("GET key".to_string()).unwrap();

        assert_eq!(result, "1");
    }

    #[test]
    fn stores_bare_floats_as_floats() {
        let scuver = new_scuver();

        scuver.run("SET key 1.5".to_string()).unwrap();
        let result = scuver.run("GET key".to_string()).unwrap();

        assert_eq!(result, "1.5");
    }

    #[test]
    fn stores_bare_booleans_as_booleans() {
        let scuver = new_scuver();

        scuver.run("SET key true".to_string()).unwrap();
        let result = scuver.run("GET key".to_string()).unwrap();

        assert_eq!(result, "true");
    }

    #[test]
    fn rejects_unquoted_strings() {
        let scuver = new_scuver();

        let result = scuver.run("SET key value".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn supports_spaces_inside_quoted_strings() {
        let scuver = new_scuver();

        scuver
            .run(r#"SET greeting "hello world""#.to_string())
            .unwrap();
        let result = scuver.run("GET greeting".to_string()).unwrap();

        assert_eq!(result, "hello world");
    }

    #[test]
    fn returns_error_for_empty_code() {
        let scuver = new_scuver();

        let result = scuver.run("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_invalid_command() {
        let scuver = new_scuver();

        let result = scuver.run("INVALID".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn get_missing_key_returns_empty_string() {
        let scuver = new_scuver();

        let result = scuver.run("GET missing".to_string()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn ignores_extra_whitespace_around_command() {
        let scuver = new_scuver();

        scuver
            .run(r#"   SET   key   "value"   "#.to_string())
            .unwrap();
        let result = scuver.run(" GET key ".to_string()).unwrap();

        assert_eq!(result, "value");
    }

    #[test]
    fn rejects_invalid_set_arity() {
        let scuver = new_scuver();

        let result = scuver.run("SET only_key".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn executes_from_file() {
        let scuver = Scuver::load("test.scv");
        println!("{:#?}", scuver);
        assert_eq!(scuver.is_ok(), true);
    }
}
