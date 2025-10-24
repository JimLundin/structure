use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Typed identifier for table records
#[derive(Debug, Serialize, Deserialize)]
pub struct Id<T> {
    value: u64,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

// Manual implementations to avoid unnecessary trait bounds on T
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Id<T> {}

impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
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

    /// Get the raw ID value (alias for value())
    pub fn as_u64(&self) -> u64 {
        self.value
    }
}

impl<T> From<u64> for Id<T> {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
