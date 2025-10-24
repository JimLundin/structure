# Struct-DB

A lightweight, in-process database library for Rust that provides type-safe struct serialization and querying.

## Overview

Struct-DB allows you to store and query Rust structs with minimal boilerplate. It's designed for applications that need persistent storage for structured data without the overhead of a full database system.

### Key Features

- **Type-safe API**: All operations are type-checked at compile time
- **Zero-overhead abstractions**: Native Rust API with builder pattern for queries
- **Persistent storage**: Write-Ahead Log (WAL) for durability
- **Concurrent access**: Multi-reader/single-writer concurrency per table
- **Ergonomic**: Single derive macro to make structs storable
- **Relationships**: Support for references between structs via `Ref<T>`

## Quick Start

### Basic Usage

```rust
use serde::{Serialize, Deserialize};
use struct_db::{Database, Table, Id, Ref};

// Define your structs
#[derive(Table, Serialize, Deserialize, Clone)]
struct User {
    name: String,
    age: u32,
}

fn main() -> anyhow::Result<()> {
    // Open database
    let db = Database::open("./data")?;

    // Register types (required for WAL replay)
    db.register_type::<User>();

    // Insert
    let user_id = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
    })?;

    // Query
    let users = db.query::<User>()
        .filter(|u| u.age > 25)
        .sort_by(|u| u.name.clone())
        .limit(10)
        .collect();

    // Update
    db.update(user_id, |user| {
        user.age = 31;
    })?;

    // Delete
    db.delete(user_id)?;

    Ok(())
}
```

### Relationships

```rust
#[derive(Table, Serialize, Deserialize, Clone)]
struct Post {
    title: String,
    content: String,
    author: Ref<User>,  // Reference to User
}

// Create post with reference to user
let post_id = db.insert(Post {
    title: "Hello World".to_string(),
    content: "My first post".to_string(),
    author: Ref::new(user_id),
})?;

// Access referenced data
let (_, post) = db.query::<Post>().first().unwrap();
let author = post.author.get(&db)?;
println!("Author: {}", author.name);
```

## Architecture

### Core Components

- **Database**: Main entry point, manages all tables and persistence
- **Table<T>**: In-memory storage for a specific struct type
- **Query<T>**: Type-safe query builder with filtering and sorting
- **Id<T>**: Typed identifier for records (auto-generated)
- **Ref<T>**: Reference to another table's record (foreign key)
- **WAL**: Write-Ahead Log for persistent storage

### Persistence Model

All operations are written to a Write-Ahead Log (WAL):

```
data/
└── wal.log     # Append-only operation log
```

- **On insert/update/delete**: Operation is appended to WAL
- **On startup**: WAL is replayed to rebuild in-memory state
- **On compact**: WAL is rewritten with current state only

### Compaction

```rust
// Compact WAL to reduce size
db.compact()?;
```

Compaction rewrites the WAL to contain only the current state, removing historical operations.

## Query API

The query API provides a fluent, type-safe interface:

```rust
let results = db.query::<User>()
    .filter(|u| u.age > 18)           // Multiple filters can be chained
    .filter(|u| u.name.starts_with("A"))
    .sort_by(|u| u.name.clone())      // Sort by field
    .limit(10)                         // Limit results
    .collect();                        // Execute query

// Get first result
let first = db.query::<User>()
    .filter(|u| u.age > 18)
    .first();

// Count matching records
let count = db.query::<User>()
    .filter(|u| u.age > 18)
    .count();
```

## Concurrency

Each table uses `RwLock` for concurrency control:
- Multiple concurrent readers
- Single writer (exclusive access)
- Lock granularity is per-table, not per-database

## ID Management

IDs are automatically generated and managed by the database:

```rust
// Insert returns an ID
let id: Id<User> = db.insert(user)?;

// IDs are typed - won't confuse User IDs with Post IDs
let user_id: Id<User> = ...;
let post_id: Id<Post> = ...;  // Different type!

// Access ID value
println!("ID: {}", id.value());
```

## Design Philosophy

1. **Rust-first**: Designed for Rust, not a FFI-friendly abstraction
2. **Type safety**: Leverage Rust's type system for correctness
3. **Simplicity**: No SQL, no ORM complexity - just native Rust APIs
4. **In-process**: Optimized for single-process, multi-threaded access
5. **Embeddable**: Minimal dependencies, easy to embed in applications

## Limitations

### Current Version (v0.1)

- **No schema evolution**: Adding/removing fields requires manual migration
- **No aggregations**: Only CRUD operations and filtering
- **No joins**: Must manually resolve relationships via `Ref<T>.get()`
- **No indexing**: All queries are full table scans
- **Single file**: All data in one WAL file
- **Size limit**: Designed for single-digit gigabytes of data

### Future Enhancements

- Schema versioning and migration
- Indexing for faster queries
- Lazy reference resolution (transparent dereferencing)
- Batch operations
- Transactions
- Compression

## Examples

See the `struct-db/examples/` directory for complete examples demonstrating various use cases:

### Available Examples

1. **`basic_crud.rs`** - Introduction to basic operations
   - Creating, reading, updating, and deleting records
   - Simple queries with filters and sorting
   - Error handling and data persistence
   - Perfect starting point for new users

2. **`sensor_data.rs`** - IoT sensor monitoring system
   - Sensor readings with timestamps
   - References between readings and sensors
   - Alerts based on sensor data
   - Querying time-series data

3. **`ecommerce.rs`** - E-commerce platform
   - Customers, products, orders, and order items
   - Complex relationships (many-to-many)
   - Aggregations (revenue, popular products)
   - Customer analytics and inventory management

4. **`blog.rs`** - Blog publishing platform
   - Authors, posts, comments, and tags
   - Nested relationships (comments → posts → authors)
   - Many-to-many relationships via join tables
   - Complex queries (popular posts, author activity)

### Running Examples

Run any example with:
```bash
cd struct-db
cargo run --example <example_name>
```

For instance:
```bash
cargo run --example basic_crud
cargo run --example sensor_data
cargo run --example ecommerce
cargo run --example blog
```

Each example creates its own database in `./data/<example_name>/` and demonstrates different aspects of the library.

## License

MIT License - see LICENSE file for details

## Contributing

Contributions welcome! This is an early-stage project and there's plenty of room for improvement.

## Acknowledgments

Inspired by:
- SQLite (embedded database philosophy)
- Sled (Rust-native embedded database)
- IndexedDB (structured storage API)
