use crate::{Database, Id, TableType};
use serde::{Deserialize, Serialize};

/// Reference to another table record
///
/// In queries, references can be eagerly loaded using `.with_refs()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ref<T> {
    id: Id<T>,
}

impl<T> Ref<T> {
    /// Create a new reference from an ID
    pub fn new(id: Id<T>) -> Self {
        Self { id }
    }

    /// Get the referenced ID
    pub fn id(&self) -> Id<T> {
        self.id
    }

    /// Fetch the referenced record from the database
    pub fn get(&self, db: &Database) -> crate::Result<T>
    where
        T: TableType + Clone,
    {
        db.get(self.id)
    }
}

impl<T> From<Id<T>> for Ref<T> {
    fn from(id: Id<T>) -> Self {
        Self::new(id)
    }
}

impl<T> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Ref<T> {}
