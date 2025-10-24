mod database;
mod id;
mod reference;
mod table;
mod query;
mod wal;
mod serialization;

pub use database::Database;
pub use id::Id;
pub use reference::Ref;
pub use query::Query;
pub use struct_db_derive::Table;

// Re-export for derive macro
#[doc(hidden)]
pub use serde;
#[doc(hidden)]
pub use bincode;

/// Trait implemented by the derive macro for all table types
pub trait TableType: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static {
    /// Unique type identifier for serialization
    fn type_name() -> &'static str;
}

/// Result type for database operations
pub type Result<T> = anyhow::Result<T>;
