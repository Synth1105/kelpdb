use std::any::Any;
use std::sync::Arc;
use ketheler::agent;

agent!(Vec<Row>);

pub struct DB {
    handle: ketheler::server::ServerHandle<Agent>
}

impl DB {
    pub fn new(name: impl Into<String>, initial: impl WriteValue) -> Self {
        let handle = Agent::start_link();
        Agent::update(&handle, vec![Row::init(name, initial)]);
        Self {
            handle
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl WriteValue) {
        let key = key.into();
        let value = value.write();
        let mut rows = Agent::get(&self.handle, |v| v);
        if let Some(row) = rows.iter_mut().find(|r| r.name() == key) {
            row.data.push(value);
        }
        Agent::update(&self.handle, rows);
    }

    pub fn remove(&mut self, key: impl Into<String>) -> Option<Arc<dyn Any + Send + Sync>> {
        let key = key.into();
        let mut rows = Agent::get(&self.handle, |v| v);
        let result = rows.iter()
            .find(|r| r.name() == key)
            .and_then(|r| r.data.last().cloned());
        if let Some(row) = rows.iter_mut().find(|r| r.name() == key) {
            row.data.pop();
        }
        Agent::update(&self.handle, rows);
        result
    }

    pub fn get(&self, key: impl Into<String>) -> Vec<Arc<dyn Any + Send + Sync>> {
        let rows = Agent::get(&self.handle, |v| v);
        let key = key.into();
        rows.iter()
            .find(|r| r.name() == key)
            .map(|r| r.data.clone())
            .unwrap_or_default()
    }

    pub fn get_by_type<T: Clone + 'static>(&self, key: impl Into<String>) -> Vec<T> {
        let rows = Agent::get(&self.handle, |v| v);
        let key = key.into();
        rows.iter()
            .find(|r| r.name() == key)
            .map(|r| {
                r.data.iter()
                    .filter_map(|v| v.downcast_ref::<T>().cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn add_row(&mut self, name: impl Into<String>, initial: impl WriteValue) {
        let mut rows = Agent::get(&self.handle, |v| v);
        rows.push(Row::init(name, initial));
        Agent::update(&self.handle, rows);
    }

    pub fn remove_row(&mut self, name: impl Into<String>) -> Option<Row> {
        let key = name.into();
        let mut rows = Agent::get(&self.handle, |v| v);
        let result = rows.iter()
            .find(|r| r.name() == key)
            .cloned();
        if let Some(pos) = rows.iter().position(|r| r.name() == key) {
            rows.remove(pos);
        }
        Agent::update(&self.handle, rows);
        result
    }
}

#[derive(Debug, Clone, Default)]
pub struct Row {
    name: String,
    data: Vec<Arc<dyn Any + Send + Sync>>
}

impl Row {
    pub fn init(name: impl Into<String>, initial: impl WriteValue) -> Self {
        Self {
            name: name.into(),
            data: vec![Arc::new(initial.write())]
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

pub trait WriteValue: Send + Sync + 'static {
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

impl_write_value! { i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 bool char }