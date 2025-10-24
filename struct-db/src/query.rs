use crate::{Id, TableType};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Query builder for type-safe filtering and sorting
pub struct Query<T: TableType> {
    data: Arc<RwLock<TableData<T>>>,
    filters: Vec<Box<dyn Fn(&T) -> bool + Send + Sync>>,
    sort_fn: Option<Box<dyn Fn(&T, &T) -> std::cmp::Ordering + Send + Sync>>,
    limit: Option<usize>,
}

// Internal table data structure (shared with Table)
pub(crate) struct TableData<T> {
    pub(crate) records: HashMap<u64, T>,
    pub(crate) next_id: u64,
}

impl<T: TableType> Query<T> {
    pub(crate) fn new(data: Arc<RwLock<TableData<T>>>) -> Self {
        Self {
            data,
            filters: Vec::new(),
            sort_fn: None,
            limit: None,
        }
    }

    /// Add a filter predicate
    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.filters.push(Box::new(predicate));
        self
    }

    /// Set a sort function
    pub fn sort_by<F>(mut self, compare: F) -> Self
    where
        F: Fn(&T) -> String + Send + Sync + 'static,
    {
        self.sort_fn = Some(Box::new(move |a, b| compare(a).cmp(&compare(b))));
        self
    }

    /// Set a custom comparison function
    pub fn sort_by_cmp<F>(mut self, compare: F) -> Self
    where
        F: Fn(&T, &T) -> std::cmp::Ordering + Send + Sync + 'static,
    {
        self.sort_fn = Some(Box::new(compare));
        self
    }

    /// Limit the number of results
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Collect results as (Id, T) pairs
    pub fn collect(self) -> Vec<(Id<T>, T)>
    where
        T: Clone,
    {
        let data = self.data.read();
        let mut results: Vec<(Id<T>, T)> = data
            .records
            .iter()
            .filter(|(_, record)| {
                self.filters.iter().all(|f| f(record))
            })
            .map(|(id, record)| (Id::new(*id), record.clone()))
            .collect();

        if let Some(sort_fn) = &self.sort_fn {
            results.sort_by(|(_, a), (_, b)| sort_fn(a, b));
        }

        if let Some(limit) = self.limit {
            results.truncate(limit);
        }

        results
    }

    /// Get the first result
    pub fn first(self) -> Option<(Id<T>, T)>
    where
        T: Clone,
    {
        self.limit(1).collect().into_iter().next()
    }

    /// Count matching records
    pub fn count(self) -> usize {
        let data = self.data.read();
        data.records
            .values()
            .filter(|record| self.filters.iter().all(|f| f(record)))
            .count()
    }
}
