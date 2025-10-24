use crate::error::{Error, Result};
use crate::{Id, Query, TableType};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// In-memory table storage for a specific type
pub struct Table<T: TableType> {
    data: Arc<RwLock<TableData<T>>>,
}

pub(crate) struct TableData<T> {
    pub(crate) records: HashMap<u64, T>,
    pub(crate) next_id: u64,
}

impl<T: TableType> Table<T> {
    pub(crate) fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(TableData {
                records: HashMap::new(),
                next_id: 1,
            })),
        }
    }

    /// Insert a new record and return its ID
    pub fn insert(&self, record: T) -> Id<T> {
        let mut data = self.data.write();
        let id = data.next_id;
        data.next_id += 1;
        data.records.insert(id, record);
        Id::new(id)
    }

    /// Get a record by ID
    pub fn get(&self, id: Id<T>) -> Option<T>
    where
        T: Clone,
    {
        let data = self.data.read();
        data.records.get(&id.value()).cloned()
    }

    /// Get a reference to a record by ID (avoids cloning)
    pub fn get_ref(&self, id: Id<T>) -> Option<parking_lot::MappedRwLockReadGuard<'_, T>> {
        let guard = self.data.read();
        if guard.records.contains_key(&id.value()) {
            Some(parking_lot::RwLockReadGuard::map(guard, |data| {
                data.records.get(&id.value()).unwrap()
            }))
        } else {
            None
        }
    }

    /// Update a record by ID
    pub fn update<F>(&self, id: Id<T>, f: F) -> Result<()>
    where
        F: FnOnce(&mut T),
    {
        let mut data = self.data.write();
        if let Some(record) = data.records.get_mut(&id.value()) {
            f(record);
            Ok(())
        } else {
            Err(Error::RecordNotFound(id.value()))
        }
    }

    /// Delete a record by ID
    pub fn delete(&self, id: Id<T>) -> Result<()> {
        let mut data = self.data.write();
        if data.records.remove(&id.value()).is_some() {
            Ok(())
        } else {
            Err(Error::RecordNotFound(id.value()))
        }
    }

    /// Create a new query builder
    pub fn query(&self) -> Query<T> {
        Query::new(self.data.clone())
    }

    /// Get all records as (Id, T) pairs
    pub fn all(&self) -> Vec<(Id<T>, T)>
    where
        T: Clone,
    {
        let data = self.data.read();
        data.records
            .iter()
            .map(|(id, record)| (Id::new(*id), record.clone()))
            .collect()
    }

    /// Insert a record with a specific ID (used for WAL replay)
    pub(crate) fn insert_with_id(&self, id: u64, record: T) {
        let mut data = self.data.write();
        data.records.insert(id, record);
        if id >= data.next_id {
            data.next_id = id + 1;
        }
    }

    /// Get the current maximum ID (for WAL operations)
    pub(crate) fn max_id(&self) -> u64 {
        let data = self.data.read();
        data.next_id.saturating_sub(1)
    }
}

impl<T: TableType> Clone for Table<T> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}
