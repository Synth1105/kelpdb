//! Shared storage primitives used by the database agent.

use ketheler::agent;
use std::any::Any;
use std::sync::Arc;

agent!(Vec<Row>);

/// A named row stored inside the agent state.
#[derive(Debug, Clone, Default)]
pub struct Row {
    /// Row identifier.
    pub name: String,
    /// Values appended to this row in insertion order.
    pub data: Vec<Arc<dyn Any + Send + Sync>>,
}

impl Row {
    /// Creates a row with a single initial value.
    pub fn init(name: impl Into<String>, initial: impl WriteValue) -> Self {
        Self {
            name: name.into(),
            data: vec![initial.write()],
        }
    }

    /// Returns the row name.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

/// Converts supported Rust values into the shared dynamic storage format.
pub trait WriteValue: Send + Sync + 'static {
    /// Boxes a value into the database storage representation.
    fn write(self) -> Arc<dyn Any + Send + Sync>;
}

macro_rules! impl_write_value {
    ($($t:ty)*) => {
        $(
            impl WriteValue for $t {
                fn write(self) -> Arc<dyn Any + Send + Sync> {
                    Arc::new(self)
                }
            }
        )*
    }
}

impl WriteValue for String {
    fn write(self) -> Arc<dyn Any + Send + Sync> {
        Arc::new(self)
    }
}

impl WriteValue for &'static str {
    fn write(self) -> Arc<dyn Any + Send + Sync> {
        Arc::new(self)
    }
}

impl_write_value! { i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 bool char }
