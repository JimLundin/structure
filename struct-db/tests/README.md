# Struct-DB Test Suite

This directory contains a comprehensive test suite for the struct-db library, covering all major functionality and edge cases.

## Test Organization

### Integration Tests

The test suite is organized into several test files, each focusing on a specific area of functionality:

#### `crud_tests.rs`
Tests for basic Create, Read, Update, and Delete operations:
- Insert and retrieve records
- Insert multiple records with unique IDs
- Update existing records
- Delete records
- Error handling for nonexistent records
- Multiple table support
- ID sequencing

#### `query_tests.rs`
Tests for the query API:
- Query all records
- Filter operations (single and chained)
- Sorting (by field, by custom comparison)
- Limiting results
- Combined filter + sort + limit
- `first()` and `count()` operations
- Empty result handling
- Complex filtering logic

#### `persistence_tests.rs`
Tests for persistence and WAL replay:
- Data persistence across database restarts
- Multiple operations persistence
- Update persistence
- Delete persistence
- Multiple tables persistence
- WAL compaction
- Compaction correctness
- Query operations after reload

#### `reference_tests.rs`
Tests for cross-table references:
- Simple references between tables
- Multiple references to the same record
- Nested references (comments → posts → users)
- Reference to deleted records (error handling)
- Reference persistence
- Querying with references
- Updating referenced records
- Circular references

#### `concurrency_tests.rs`
Tests for concurrent access:
- Concurrent reads
- Concurrent inserts
- Concurrent updates
- Mixed concurrent operations
- Concurrent queries
- Concurrent deletes and reads
- Concurrent queries and inserts

#### `unit_tests.rs`
Unit tests for core types and edge cases:
- **Id type**: Creation, equality, type safety, display, serialization, copying, hashing
- **Ref type**: Creation, ID retrieval, serialization, dereferencing, error handling
- **Edge cases**: Empty database, empty strings, Unicode strings, very long strings, special characters, large numbers of records, multiple updates, delete and reinsert

#### `wal_tests.rs`
Tests for Write-Ahead Log functionality:
- WAL file creation
- WAL growth with operations
- Recording all operations
- WAL replay on restart
- WAL replay order preservation
- WAL replay with deletes
- WAL compaction
- Multiple restart survival
- Many operations handling
- Large records
- Operations after compaction

## Running the Tests

### Run All Tests
```bash
cargo test
```

### Run Tests from a Specific File
```bash
cargo test --test crud_tests
cargo test --test query_tests
cargo test --test persistence_tests
cargo test --test reference_tests
cargo test --test concurrency_tests
cargo test --test unit_tests
cargo test --test wal_tests
```

### Run a Specific Test
```bash
cargo test test_insert_and_get
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Tests in Parallel (default)
```bash
cargo test
```

### Run Tests Sequentially
```bash
cargo test -- --test-threads=1
```

## Test Coverage

The test suite provides comprehensive coverage of:

1. **CRUD Operations** - All basic database operations
2. **Query API** - Filtering, sorting, limiting, counting
3. **Persistence** - WAL-based durability and recovery
4. **References** - Cross-table relationships and foreign keys
5. **Concurrency** - Multi-threaded access patterns
6. **Type Safety** - Compile-time guarantees for IDs and references
7. **Edge Cases** - Empty values, Unicode, large data, error conditions
8. **WAL Internals** - Log file operations and compaction

## Test Data

All tests use `tempfile::TempDir` to create isolated temporary directories for database files. This ensures:
- No test pollution between runs
- Automatic cleanup after tests
- Parallel test execution safety

## Dependencies

The test suite requires the following dev-dependencies:
- `tempfile` - For creating temporary test directories

## Test Principles

1. **Isolation** - Each test uses its own temporary database
2. **Clarity** - Test names clearly describe what they test
3. **Coverage** - Tests cover both happy paths and error cases
4. **Concurrency** - Tests verify thread-safe operations
5. **Persistence** - Tests verify data survives restarts
6. **Documentation** - Tests serve as usage examples

## Continuous Integration

These tests should be run in CI/CD pipelines to ensure:
- All tests pass before merging
- No regressions are introduced
- Performance characteristics are maintained

## Adding New Tests

When adding new functionality to struct-db:
1. Add unit tests for new types/functions
2. Add integration tests for new features
3. Add edge case tests for error handling
4. Add concurrency tests if relevant
5. Update this README with new test categories
