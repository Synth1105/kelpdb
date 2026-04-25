//! `kelpdb` is a small in-memory database whose storage is backed by a
//! `ketheler` agent.
//!
//! The main entry point is [`db::DB`]. Use [`prelude`] for the common imports.

pub mod db;
pub mod prelude;
#[cfg(feature = "scuver")]
pub mod scuver;
pub mod utils;
