use crate::serialization::WalEntry;
use crate::wal::Wal;
use crate::{Id, Query, TableType};
use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Main database instance
pub struct Database {
    path: PathBuf,
    tables: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
    wal: Arc<Wal>,
}

impl Database {
    /// Open or create a database at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;

        let wal_path = path.join("wal.log");
        let wal = Wal::open(&wal_path)?;

        let db = Self {
            path,
            tables: Arc::new(RwLock::new(HashMap::new())),
            wal: Arc::new(wal),
        };

        // Replay WAL to restore state
        db.replay_wal()?;

        Ok(db)
    }

    /// Get or create a table for type T
    fn get_table<T: TableType>(&self) -> Arc<RwLock<crate::query::TableData<T>>> {
        let type_id = TypeId::of::<T>();
        let mut tables = self.tables.write();

        if !tables.contains_key(&type_id) {
            let table_data = Arc::new(RwLock::new(crate::query::TableData {
                records: HashMap::new(),
                next_id: 1,
            }));
            tables.insert(type_id, Box::new(table_data.clone()));
            table_data
        } else {
            tables
                .get(&type_id)
                .unwrap()
                .downcast_ref::<Arc<RwLock<crate::query::TableData<T>>>>()
                .unwrap()
                .clone()
        }
    }

    /// Insert a new record
    pub fn insert<T: TableType>(&self, record: T) -> crate::Result<Id<T>> {
        let table = self.get_table::<T>();
        let mut data = table.write();

        let id = data.next_id;
        data.next_id += 1;

        // Write to WAL
        let entry = WalEntry::insert(T::type_name(), id, &record)?;
        self.wal.append(&entry)?;

        // Insert into table
        data.records.insert(id, record);

        Ok(Id::new(id))
    }

    /// Get a record by ID
    pub fn get<T: TableType>(&self, id: Id<T>) -> crate::Result<&T> {
        // Note: This signature is problematic - we can't return a reference that outlives the lock
        // For now, we'll use a different approach
        Err(anyhow::anyhow!("get() not yet implemented - use query() instead"))
    }

    /// Get a record by ID (returns a clone)
    pub fn get_cloned<T: TableType + Clone>(&self, id: Id<T>) -> crate::Result<T> {
        let table = self.get_table::<T>();
        let data = table.read();

        data.records
            .get(&id.value())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Record not found: {}", id.value()))
    }

    /// Update a record by ID
    pub fn update<T: TableType, F>(&self, id: Id<T>, f: F) -> crate::Result<()>
    where
        F: FnOnce(&mut T),
    {
        let table = self.get_table::<T>();
        let mut data = table.write();

        if let Some(record) = data.records.get_mut(&id.value()) {
            f(record);

            // Write to WAL
            let entry = WalEntry::update(T::type_name(), id.value(), record)?;
            self.wal.append(&entry)?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("Record not found: {}", id.value()))
        }
    }

    /// Delete a record by ID
    pub fn delete<T: TableType>(&self, id: Id<T>) -> crate::Result<()> {
        let table = self.get_table::<T>();
        let mut data = table.write();

        if data.records.remove(&id.value()).is_some() {
            // Write to WAL
            let entry = WalEntry::delete(T::type_name(), id.value());
            self.wal.append(&entry)?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("Record not found: {}", id.value()))
        }
    }

    /// Create a query for type T
    pub fn query<T: TableType>(&self) -> Query<T> {
        let table = self.get_table::<T>();
        Query::new(table)
    }

    /// Get all records for type T
    pub fn all<T: TableType + Clone>(&self) -> Vec<(Id<T>, T)> {
        self.query::<T>().collect()
    }

    /// Compact the WAL by rewriting it with current state only
    pub fn compact(&self) -> crate::Result<()> {
        let mut entries = Vec::new();

        // Collect all current records as Insert entries
        let tables = self.tables.read();
        for (_type_id, table_any) in tables.iter() {
            // We need to serialize each table's data
            // This is tricky because we've lost type information
            // For now, we'll skip this and add it later
        }

        // For simplicity, we'll read the WAL and compact it
        // A better approach would be to iterate through all tables
        let all_entries = self.wal.read_all()?;

        // Keep only the latest operation for each (type, id) pair
        let mut latest: HashMap<(String, u64), WalEntry> = HashMap::new();

        for entry in all_entries {
            let key = match &entry {
                WalEntry::Insert { type_name, id, .. } => (type_name.clone(), *id),
                WalEntry::Update { type_name, id, .. } => (type_name.clone(), *id),
                WalEntry::Delete { type_name, id } => (type_name.clone(), *id),
            };

            latest.insert(key, entry);
        }

        // Remove deletes and convert updates to inserts
        for entry in latest.values() {
            match entry {
                WalEntry::Delete { .. } => {
                    // Skip deletes - the record doesn't exist anymore
                }
                WalEntry::Update { type_name, id, data } => {
                    entries.push(WalEntry::Insert {
                        type_name: type_name.clone(),
                        id: *id,
                        data: data.clone(),
                    });
                }
                WalEntry::Insert { .. } => {
                    entries.push(entry.clone());
                }
            }
        }

        self.wal.compact(&entries)?;
        Ok(())
    }

    /// Replay WAL to restore database state
    fn replay_wal(&self) -> crate::Result<()> {
        let entries = self.wal.read_all()?;

        for entry in entries {
            self.replay_entry(entry)?;
        }

        Ok(())
    }

    /// Replay a single WAL entry
    fn replay_entry(&self, entry: WalEntry) -> crate::Result<()> {
        // Type registration is handled per-type
        // For now, we skip unknown types (they'll be registered when first accessed)
        Ok(())
    }

    /// Register a type for WAL replay
    pub fn register_type<T: TableType>(&self) {
        // Ensure the table exists
        self.get_table::<T>();
    }

    /// Replay a typed entry (called by user code after registration)
    pub(crate) fn replay_typed_entry<T: TableType>(
        &self,
        id: u64,
        data: &[u8],
    ) -> crate::Result<()> {
        let record: T = bincode::deserialize(data)?;
        let table = self.get_table::<T>();
        let mut table_data = table.write();

        table_data.records.insert(id, record);
        if id >= table_data.next_id {
            table_data.next_id = id + 1;
        }

        Ok(())
    }
}
