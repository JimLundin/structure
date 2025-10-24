use serde::{Deserialize, Serialize};

/// WAL operation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalEntry {
    Insert {
        type_name: String,
        id: u64,
        data: Vec<u8>,
    },
    Update {
        type_name: String,
        id: u64,
        data: Vec<u8>,
    },
    Delete {
        type_name: String,
        id: u64,
    },
}

impl WalEntry {
    pub fn insert<T: serde::Serialize>(type_name: &str, id: u64, data: &T) -> crate::Result<Self> {
        let serialized = bincode::serialize(data)?;
        Ok(WalEntry::Insert {
            type_name: type_name.to_string(),
            id,
            data: serialized,
        })
    }

    pub fn update<T: serde::Serialize>(type_name: &str, id: u64, data: &T) -> crate::Result<Self> {
        let serialized = bincode::serialize(data)?;
        Ok(WalEntry::Update {
            type_name: type_name.to_string(),
            id,
            data: serialized,
        })
    }

    pub fn delete(type_name: &str, id: u64) -> Self {
        WalEntry::Delete {
            type_name: type_name.to_string(),
            id,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            WalEntry::Insert { type_name, .. } => type_name,
            WalEntry::Update { type_name, .. } => type_name,
            WalEntry::Delete { type_name, .. } => type_name,
        }
    }
}
