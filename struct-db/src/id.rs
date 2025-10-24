use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Typed identifier for table records
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id<T> {
    value: u64,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> Id<T> {
    /// Create a new ID from a raw value
    pub(crate) fn new(value: u64) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    /// Get the raw ID value
    pub fn value(&self) -> u64 {
        self.value
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
