//! Agent-backed database implementation.

use crate::utils::{Agent, Row, WriteValue};
use ketheler::server::ServerHandle;
use std::any::Any;
use std::sync::Arc;

/// In-memory database whose rows are stored inside a `ketheler` agent.
///
/// Cloning `DB` clones the server handle, so all clones share the same state.
#[derive(Clone)]
pub struct DB {
    handle: ServerHandle<Agent>,
}

impl DB {
    /// Creates a new database and seeds it with one row/value pair.
    pub fn new(name: impl Into<String>, initial: impl WriteValue) -> Self {
        let handle = Agent::start_link();
        let mut db = Self { handle };
        db.add_row(name, initial);
        db
    }

    /// Appends a value to a row, creating the row if it does not exist.
    pub fn set(&mut self, key: impl Into<String>, value: impl WriteValue) {
        let key = key.into();
        Agent::get_and_update(&self.handle, move |mut rows| {
            if let Some(row) = rows.iter_mut().find(|row| row.name == key) {
                row.data.push(value.write());
            } else {
                rows.push(Row::init(key, value));
            }
            rows
        });
    }

    /// Removes and returns the most recently inserted value for a row.
    ///
    /// If the last value is removed, the whole row is dropped.
    pub fn remove(&mut self, key: impl Into<String>) -> Option<Arc<dyn Any + Send + Sync>> {
        let key = key.into();
        let mut rows = self.read_rows();
        let mut removed = None;

        if let Some(index) = rows.iter().position(|row| row.name == key) {
            let row = &mut rows[index];
            removed = row.data.pop();

            if row.data.is_empty() {
                rows.remove(index);
            }
        }

        self.write_rows(rows);
        removed
    }

    /// Returns the raw stored values for a row.
    pub fn get(&self, key: impl Into<String>) -> Vec<Arc<dyn Any + Send + Sync>> {
        let key = key.into();
        self.read_rows()
            .into_iter()
            .find(|row| row.name == key)
            .map(|row| row.data)
            .unwrap_or_default()
    }

    /// Returns a row stringified for CLI-facing output.
    pub fn get_display(&self, key: impl Into<String>) -> Vec<String> {
        self.get(key).iter().filter_map(stringify_value).collect()
    }

    /// Returns only the values that can be downcast to `T`.
    pub fn get_by_type<T: Clone + 'static>(&self, key: impl Into<String>) -> Vec<T> {
        self.get(key)
            .iter()
            .filter_map(|value| value.downcast_ref::<T>().cloned())
            .collect()
    }

    /// Adds a new row with an initial value, or appends to an existing row.
    pub fn add_row(&mut self, name: impl Into<String>, initial: impl WriteValue) {
        let name = name.into();
        Agent::get_and_update(&self.handle, move |mut rows| {
            if let Some(row) = rows.iter_mut().find(|row| row.name == name) {
                row.data.push(initial.write());
            } else {
                rows.push(Row::init(name, initial));
            }
            rows
        });
    }

    /// Removes and returns the entire row by name.
    pub fn remove_row(&mut self, name: impl Into<String>) -> Option<Row> {
        let key = name.into();
        let mut rows = self.read_rows();
        let removed = rows
            .iter()
            .position(|row| row.name == key)
            .map(|index| rows.remove(index));

        self.write_rows(rows);
        removed
    }

    fn read_rows(&self) -> Vec<Row> {
        Agent::get(&self.handle, |rows| rows)
    }

    fn write_rows(&self, rows: Vec<Row>) {
        Agent::update(&self.handle, rows);
    }
}

/// Converts supported value types into printable strings.
fn stringify_value(value: &Arc<dyn Any + Send + Sync>) -> Option<String> {
    macro_rules! stringify_scalar {
        ($($ty:ty),* $(,)?) => {
            $(
                if let Some(value) = value.downcast_ref::<$ty>() {
                    return Some(value.to_string());
                }
            )*
        };
    }

    if let Some(value) = value.downcast_ref::<String>() {
        return Some(value.clone());
    }

    if let Some(value) = value.downcast_ref::<&'static str>() {
        return Some((*value).to_string());
    }

    stringify_scalar!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, bool, char);
    None
}

#[cfg(test)]
mod tests {
    use super::DB;
    use crate::utils::Row;

    #[test]
    fn shares_state_across_clones_via_agent() {
        let db = DB::new("user", String::from("John"));
        let mut clone = db.clone();

        clone.set("user", 25i32);
        clone.set("user", 180.5f64);

        assert_eq!(db.get_by_type::<String>("user"), vec![String::from("John")]);
        assert_eq!(db.get_by_type::<i32>("user"), vec![25]);
        assert_eq!(db.get_by_type::<f64>("user"), vec![180.5]);
    }

    #[test]
    fn displays_string_and_scalar_values() {
        let mut db = DB::new("user", String::from("John"));

        db.set("user", 25i32);
        db.set("user", 180.5f64);

        assert_eq!(db.get_display("user"), vec!["John", "25", "180.5"]);
    }

    #[test]
    fn set_creates_a_missing_row() {
        let mut db = DB::new("seed", "value");

        db.set("created", true);

        assert_eq!(db.get_by_type::<bool>("created"), vec![true]);
    }

    #[test]
    fn remove_pops_last_value_and_keeps_remaining_values() {
        let mut db = DB::new("user", "John");

        db.set("user", 25i32);
        db.set("user", 30i32);

        let removed = db.remove("user").unwrap();

        assert_eq!(removed.downcast_ref::<i32>().copied(), Some(30));
        assert_eq!(db.get_display("user"), vec!["John", "25"]);
    }

    #[test]
    fn remove_drops_row_when_last_value_is_removed() {
        let mut db = DB::new("user", "John");

        let removed = db.remove("user").unwrap();

        assert_eq!(
            removed.downcast_ref::<&'static str>().copied(),
            Some("John")
        );
        assert!(db.get("user").is_empty());
    }

    #[test]
    fn remove_row_returns_full_row_and_clears_it() {
        let mut db = DB::new("user", "John");

        db.set("user", 25i32);
        let removed = db.remove_row("user").unwrap();

        assert_eq!(removed.name, "user");
        assert_eq!(removed.data.len(), 2);
        assert!(db.get("user").is_empty());
    }

    #[test]
    fn remove_row_returns_none_for_missing_row() {
        let mut db = DB::new("user", "John");

        assert!(db.remove_row("missing").is_none());
    }

    #[test]
    fn get_by_type_filters_out_other_types() {
        let mut db = DB::new("user", "John");

        db.set("user", 25i32);
        db.set("user", false);

        assert_eq!(db.get_by_type::<i32>("user"), vec![25]);
        assert_eq!(db.get_by_type::<bool>("user"), vec![false]);
        assert_eq!(db.get_by_type::<f64>("user"), Vec::<f64>::new());
    }

    #[test]
    fn add_row_appends_to_existing_row() {
        let mut db = DB::new("posts", "first");

        db.add_row("posts", "second");

        assert_eq!(db.get_display("posts"), vec!["first", "second"]);
    }

    #[test]
    fn get_returns_empty_for_missing_row() {
        let db = DB::new("user", "John");

        assert!(db.get("missing").is_empty());
    }

    #[test]
    fn remove_returns_none_for_missing_row() {
        let mut db = DB::new("user", "John");

        assert!(db.remove("missing").is_none());
    }

    #[test]
    fn remove_row_preserves_row_shape() {
        let mut db = DB::new("posts", "first");

        db.add_row("posts", "second");
        let removed = db.remove_row("posts").unwrap();

        assert_eq!(
            removed
                .data
                .iter()
                .filter_map(|value| value.downcast_ref::<&'static str>().copied())
                .collect::<Vec<_>>(),
            vec!["first", "second"]
        );
    }

    #[test]
    fn row_init_stores_the_initial_value() {
        let row = Row::init("user", "John");

        assert_eq!(row.name(), "user");
        assert_eq!(row.data.len(), 1);
        assert_eq!(
            row.data[0].downcast_ref::<&'static str>().copied(),
            Some("John")
        );
    }
}
