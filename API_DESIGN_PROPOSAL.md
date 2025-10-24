# API Design Proposals for struct-db

## 1. Type Registration - Compile-Time Enforcement

### Current Problem
```rust
let db = Database::open("./data")?;
db.register_type::<User>();  // Easy to forget, silent failure
```

### Proposed Solution: Type-Safe Builder Pattern

```rust
// User must register types at DB creation time
let db = Database::open("./data")?
    .register::<User>()
    .register::<Post>()
    .register::<Comment>()
    .build()?;

// Or with macro for convenience:
let db = Database::open("./data")?
    .with_types!(User, Post, Comment)?;

// Now any query for unregistered type fails at runtime with clear error
db.query::<UnregisteredType>() // Error: "Type 'UnregisteredType' not registered"
```

### Implementation Strategy

1. **Track registered types in Database struct:**
```rust
pub struct Database {
    path: PathBuf,
    tables: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
    registered_types: Arc<RwLock<HashSet<TypeId>>>, // NEW
    wal: Arc<Wal>,
}
```

2. **Check on every operation:**
```rust
pub fn query<T: TableType>(&self) -> crate::Result<Query<T>> {
    let type_id = TypeId::of::<T>();
    let registered = self.registered_types.read();

    if !registered.contains(&type_id) {
        return Err(Error::TypeNotRegistered(T::type_name().to_string()));
    }

    let table = self.get_table::<T>();
    Ok(Query::new(table))
}
```

3. **Builder pattern for database creation:**
```rust
pub struct DatabaseBuilder {
    path: PathBuf,
    registered_types: HashSet<TypeId>,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> crate::Result<DatabaseBuilder> {
        Ok(DatabaseBuilder {
            path: path.as_ref().to_path_buf(),
            registered_types: HashSet::new(),
        })
    }
}

impl DatabaseBuilder {
    pub fn register<T: TableType>(mut self) -> Self {
        self.registered_types.insert(TypeId::of::<T>());
        self
    }

    pub fn build(self) -> crate::Result<Database> {
        // Create database, replay WAL with registered types
        // ...
    }
}
```

### Alternative: Self-Registering Types with `inventory` crate

```rust
// In derive macro, generate:
inventory::submit! {
    TypeRegistration::new::<User>()
}

// Database automatically discovers all types at startup
let db = Database::open("./data")?; // Auto-discovers all Table types
```

**Pros:** Zero boilerplate
**Cons:** Global registry, less explicit, harder to reason about

## 2. Sorting Without Cloning

### Current Problem
```rust
db.query::<User>()
    .sort_by(|u| u.name.clone())  // Must clone!
```

### Proposed Solution

```rust
impl<T: TableType> Query<T> {
    /// Sort by a borrowed key (no cloning)
    pub fn sort_by_key<K, F>(mut self, key_fn: F) -> Self
    where
        F: Fn(&T) -> &K + Send + Sync + 'static,
        K: Ord + ?Sized,
    {
        self.sort_fn = Some(Box::new(move |a, b| {
            key_fn(a).cmp(key_fn(b))
        }));
        self
    }

    /// Sort by a computed value (allows cloning if needed)
    pub fn sort_by_computed<K, F>(mut self, compute_fn: F) -> Self
    where
        F: Fn(&T) -> K + Send + Sync + 'static,
        K: Ord,
    {
        self.sort_fn = Some(Box::new(move |a, b| {
            compute_fn(a).cmp(&compute_fn(b))
        }));
        self
    }
}

// Usage:
db.query::<User>()
    .sort_by_key(|u| &u.name)  // No clone needed!

db.query::<User>()
    .sort_by_computed(|u| u.age * 2)  // For computed values
```

## 3. Atomic Compaction

### Current Problem
- Unclear semantics
- No guarantee of atomicity
- Unknown behavior during failures

### Proposed Implementation

```rust
impl Database {
    /// Compact the WAL atomically.
    ///
    /// This operation:
    /// - Blocks all writes (acquires exclusive lock)
    /// - Allows concurrent reads
    /// - Is atomic (writes to temp file, then atomic rename)
    /// - Is crash-safe (old WAL preserved until success)
    pub fn compact(&self) -> crate::Result<()> {
        // 1. Create temp file
        let temp_path = self.path.join("wal.log.compacting");

        // 2. Acquire write lock on WAL (blocks other writes)
        let _write_guard = self.wal.lock_for_compaction();

        // 3. Collect current state from all tables
        let entries = self.collect_current_state()?;

        // 4. Write to temp file
        self.wal.write_temp(&temp_path, &entries)?;

        // 5. Atomic rename (crash-safe)
        std::fs::rename(&temp_path, self.path.join("wal.log"))?;

        // 6. Lock released automatically
        Ok(())
    }

    /// Compact in background (non-blocking)
    pub fn compact_async(&self) -> CompactionHandle {
        let db = self.clone(); // Arc clone
        let handle = std::thread::spawn(move || {
            db.compact()
        });
        CompactionHandle { handle }
    }
}

pub struct CompactionHandle {
    handle: std::thread::JoinHandle<crate::Result<()>>,
}

impl CompactionHandle {
    pub fn wait(self) -> crate::Result<()> {
        self.handle.join().unwrap()
    }
}
```

## 4. Batch Operations

### Proposed API

```rust
impl Database {
    /// Insert multiple records in a single WAL transaction
    pub fn insert_batch<T: TableType>(&self, records: Vec<T>) -> crate::Result<Vec<Id<T>>> {
        let table = self.get_table::<T>();
        let mut data = table.write();
        let mut ids = Vec::with_capacity(records.len());

        // Generate all IDs first
        let start_id = data.next_id;
        for (i, record) in records.into_iter().enumerate() {
            let id = start_id + i as u64;
            ids.push(Id::new(id));
            data.records.insert(id, record);
        }
        data.next_id = start_id + ids.len() as u64;

        // Single WAL write for entire batch
        let entry = WalEntry::insert_batch(T::type_name(), &ids, &data.records)?;
        self.wal.append(&entry)?;

        Ok(ids)
    }

    /// Update multiple records
    pub fn update_batch<T: TableType, F>(&self, ids: &[Id<T>], f: F) -> crate::Result<()>
    where
        F: Fn(&mut T),
    {
        // Similar implementation
    }

    /// Delete multiple records
    pub fn delete_batch<T: TableType>(&self, ids: &[Id<T>]) -> crate::Result<()> {
        // Similar implementation
    }
}
```

## 5. UUID vs Sequential IDs

### Analysis

#### Sequential u64 IDs (Current)
**Pros:**
- Compact: 8 bytes
- Fast comparison and hashing
- Natural ordering (useful for "recent items")
- Better cache locality
- Human-readable: "User 42" vs "User 550e8400-e29b-41d4-a716-446655440000"
- Better for range queries and indexing

**Cons:**
- Not globally unique
- Reveals insertion order (privacy concern?)
- Not suitable for distributed systems
- Sequential IDs can be predicted

#### UUIDs
**Pros:**
- Globally unique (no coordination needed)
- Can generate client-side
- Good for distributed systems
- No information leakage about count/order

**Cons:**
- Large: 16 bytes (2x storage)
- Slower comparison
- No natural ordering
- Poor cache locality (random access pattern)
- Hard to debug/read
- Overkill for embedded, single-process DB

### Recommendation

For an **embedded, single-process database**, sequential IDs are the better choice:

1. **Performance**: Sequential IDs have better cache locality and are faster
2. **Storage**: Half the size of UUIDs
3. **Debugging**: Much easier to work with "ID: 42" than "ID: 550e8400-..."
4. **Use case**: This library is for embedded use, not distributed systems

However, we should make it **configurable**:

```rust
pub trait IdGenerator: Send + Sync {
    type Id: Copy + Eq + Hash + Serialize + DeserializeOwned;
    fn next(&self) -> Self::Id;
}

pub struct SequentialIdGenerator {
    next_id: AtomicU64,
}

pub struct UuidGenerator;

impl IdGenerator for SequentialIdGenerator {
    type Id = u64;
    fn next(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl IdGenerator for UuidGenerator {
    type Id = Uuid;
    fn next(&self) -> Uuid {
        Uuid::new_v4()
    }
}

// Usage:
let db = Database::open("./data")?
    .with_id_generator(SequentialIdGenerator::new())
    .register::<User>()
    .build()?;
```

But for v0.1, **keep it simple with u64** - add UUID support later if needed.

### Middle Ground: Snowflake IDs

If you want globally unique IDs but still want the benefits of sequential IDs, consider **Snowflake IDs**:

```
| 41 bits: timestamp | 10 bits: machine ID | 12 bits: sequence |
```

- Still 64 bits
- Roughly time-ordered (good cache locality)
- Globally unique across machines
- Fast comparison

This is a good compromise for future distributed use while maintaining current benefits.

## Summary of Recommendations

1. **Registration**: Use builder pattern with runtime checks, not compile-time
2. **Sorting**: Add `sort_by_key` that takes `Fn(&T) -> &K`
3. **Compaction**: Make atomic with temp file + rename, block writes
4. **Batching**: Add `insert_batch`, etc. - good for performance
5. **IDs**: Keep u64 for now, make pluggable later (Snowflake IDs could be nice middle ground)

The key insight: This is an **embedded, single-process database**. Design for that use case first, then add distributed features later if needed. Sequential IDs are perfect for this use case.
