use std::any::Any;
use std::sync::Arc;
use ketheler::agent;

agent!(Vec<Row>);


#[derive(Debug, Clone, Default)]
pub struct Row {
    pub name: String,
    pub data: Vec<Arc<dyn Any + Send + Sync>>
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
