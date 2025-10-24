use crate::{Id, TableType};
use crate::table::TableData;
use parking_lot::RwLock;
use std::sync::Arc;

/// Query builder for type-safe filtering and sorting
pub struct Query<T: TableType> {
    data: Arc<RwLock<TableData<T>>>,
    filters: Vec<Box<dyn Fn(&T) -> bool + Send + Sync>>,
    sort_fn: Option<Box<dyn Fn(&T, &T) -> std::cmp::Ordering + Send + Sync>>,
    limit: Option<usize>,
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

    /// Sort by a key extracted from each record (no cloning needed)
    ///
    /// # Example
    /// ```no_run
    /// # use struct_db::{Database, Table};
    /// # use serde::{Serialize, Deserialize};
    /// # #[derive(Table, Serialize, Deserialize, Clone)]
    /// # struct User { name: String, age: u32 }
    /// # let db = Database::open("./data")?.register::<User>().build()?;
    /// // Sort by name without cloning
    /// let users = db.query::<User>()?
    ///     .sort_by_key(|u| &u.name)
    ///     .collect();
    ///
    /// // Sort by age
    /// let users = db.query::<User>()?
    ///     .sort_by_key(|u| &u.age)
    ///     .collect();
    /// # Ok::<(), struct_db::Error>(())
    /// ```
    pub fn sort_by_key<K, F>(mut self, key_fn: F) -> Self
    where
        F: Fn(&T) -> &K + Send + Sync + 'static,
        K: Ord + ?Sized,
    {
        self.sort_fn = Some(Box::new(move |a, b| key_fn(a).cmp(key_fn(b))));
        self
    }

    /// Set a custom comparison function for advanced sorting
    ///
    /// Use this when you need complex sorting logic that can't be expressed
    /// with `sort_by_key`.
    pub fn sort_by<F>(mut self, compare: F) -> Self
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
