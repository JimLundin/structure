// This module tests the WAL (Write-Ahead Log) implementation directly
// These are lower-level tests compared to the persistence tests

use struct_db::{Database, Table};
use serde::{Deserialize, Serialize};
use std::fs;
use tempfile::TempDir;

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct TestRecord {
    value: String,
}

#[test]
fn test_wal_file_created() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let _db = Database::open(&db_path).unwrap();

    // WAL file should be created
    let wal_path = db_path.join("wal.log");
    assert!(wal_path.exists());
}

#[test]
fn test_wal_grows_with_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();
    let wal_path = db_path.join("wal.log");

    let db = Database::open(&db_path)


        .unwrap()


        .register::<TestRecord>()


        .build()


        .unwrap();// Initial size (should be 0 or very small)
    let initial_size = fs::metadata(&wal_path).unwrap().len();

    // Insert a record
    db.insert(TestRecord {
        value: "test".to_string(),
    }).unwrap();

    // WAL should have grown
    let after_insert_size = fs::metadata(&wal_path).unwrap().len();
    assert!(after_insert_size > initial_size);
}

#[test]
fn test_wal_records_all_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();
    let wal_path = db_path.join("wal.log");

    let db = Database::open(&db_path)


        .unwrap()


        .register::<TestRecord>()


        .build()


        .unwrap();let size_after_open = fs::metadata(&wal_path).unwrap().len();

    // Insert
    let id = db.insert(TestRecord {
        value: "test1".to_string(),
    }).unwrap();

    let size_after_insert = fs::metadata(&wal_path).unwrap().len();
    assert!(size_after_insert > size_after_open);

    // Update
    db.update(id, |r: &mut TestRecord| {
        r.value = "test2".to_string();
    }).unwrap();

    let size_after_update = fs::metadata(&wal_path).unwrap().len();
    assert!(size_after_update > size_after_insert);

    // Delete
    db.delete(id).unwrap();

    let size_after_delete = fs::metadata(&wal_path).unwrap().len();
    assert!(size_after_delete > size_after_update);
}

#[test]
fn test_wal_replay_on_restart() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let id = {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<TestRecord>()

            .build()

            .unwrap();db.insert(TestRecord {
            value: "initial".to_string(),
        }).unwrap()
    }; // Database drops

    // Reopen - WAL should be replayed
    let db = Database::open(&db_path)

        .unwrap()

        .register::<TestRecord>()

        .build()

        .unwrap();let record = db.get(id).unwrap();
    assert_eq!(record.value, "initial");
}

#[test]
fn test_wal_replay_preserves_order() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let id = {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<TestRecord>()

            .build()

            .unwrap();let id = db.insert(TestRecord {
            value: "v1".to_string(),
        }).unwrap();

        db.update(id, |r: &mut TestRecord| {
            r.value = "v2".to_string();
        }).unwrap();

        db.update(id, |r: &mut TestRecord| {
            r.value = "v3".to_string();
        }).unwrap();

        id
    };

    // Reopen - should have final value
    let db = Database::open(&db_path)

        .unwrap()

        .register::<TestRecord>()

        .build()

        .unwrap();let record = db.get(id).unwrap();
    assert_eq!(record.value, "v3");
}

#[test]
fn test_wal_replay_with_deletes() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let (id1, id2) = {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<TestRecord>()

            .build()

            .unwrap();let id1 = db.insert(TestRecord {
            value: "record1".to_string(),
        }).unwrap();

        let id2 = db.insert(TestRecord {
            value: "record2".to_string(),
        }).unwrap();

        db.delete(id1).unwrap();

        (id1, id2)
    };

    // Reopen
    let db = Database::open(&db_path)

        .unwrap()

        .register::<TestRecord>()

        .build()

        .unwrap();// id1 should not exist
    assert!(db.get(id1).is_err());

    // id2 should exist
    let record2 = db.get(id2).unwrap();
    assert_eq!(record2.value, "record2");
}

#[test]
fn test_wal_compact_creates_temp_file() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let db = Database::open(&db_path)


        .unwrap()


        .register::<TestRecord>()


        .build()


        .unwrap();// Insert some data
    for i in 0..10 {
        db.insert(TestRecord {
            value: format!("record{}", i),
        }).unwrap();
    }

    // Compact - this should work without errors
    db.compact().unwrap();

    // Verify temp file doesn't exist after compaction
    let temp_path = db_path.join("wal.tmp");
    assert!(!temp_path.exists());
}

#[test]
fn test_wal_survives_multiple_restarts() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    // First session - insert
    let id = {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<TestRecord>()

            .build()

            .unwrap();db.insert(TestRecord {
            value: "v1".to_string(),
        }).unwrap()
    };

    // Second session - update
    {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<TestRecord>()

            .build()

            .unwrap();db.update(id, |r: &mut TestRecord| {
            r.value = "v2".to_string();
        }).unwrap();
    }

    // Third session - update again
    {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<TestRecord>()

            .build()

            .unwrap();db.update(id, |r: &mut TestRecord| {
            r.value = "v3".to_string();
        }).unwrap();
    }

    // Fourth session - verify
    let db = Database::open(&db_path)

        .unwrap()

        .register::<TestRecord>()

        .build()

        .unwrap();let record = db.get(id).unwrap();
    assert_eq!(record.value, "v3");
}

#[test]
fn test_wal_with_many_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let ids = {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<TestRecord>()

            .build()

            .unwrap();let mut ids = vec![];
        for i in 0..100 {
            let id = db.insert(TestRecord {
                value: format!("record{}", i),
            }).unwrap();
            ids.push(id);
        }

        // Update every other record
        for (i, &id) in ids.iter().enumerate() {
            if i % 2 == 0 {
                db.update(id, |r: &mut TestRecord| {
                    r.value = format!("updated{}", i);
                }).unwrap();
            }
        }

        // Delete every third record
        for (i, &id) in ids.iter().enumerate() {
            if i % 3 == 0 {
                db.delete(id).unwrap();
            }
        }

        ids
    };

    // Reopen and verify
    let db = Database::open(&db_path)

        .unwrap()

        .register::<TestRecord>()

        .build()

        .unwrap();for (i, &id) in ids.iter().enumerate() {
        if i % 3 == 0 {
            // Should be deleted
            assert!(db.get(id).is_err());
        } else {
            let record = db.get(id).unwrap();
            if i % 2 == 0 {
                assert_eq!(record.value, format!("updated{}", i));
            } else {
                assert_eq!(record.value, format!("record{}", i));
            }
        }
    }
}

#[test]
fn test_wal_empty_database_restart() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    // Create database but don't insert anything
    {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<TestRecord>()

            .build()

            .unwrap();}

    // Reopen
    let db = Database::open(&db_path)

        .unwrap()

        .register::<TestRecord>()

        .build()

        .unwrap();let results = db.query::<TestRecord>().unwrap().collect();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_wal_with_large_records() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let id = {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<TestRecord>()

            .build()

            .unwrap();// Insert a large record
        let large_value = "x".repeat(100_000);
        db.insert(TestRecord {
            value: large_value,
        }).unwrap()
    };

    // Reopen and verify
    let db = Database::open(&db_path)

        .unwrap()

        .register::<TestRecord>()

        .build()

        .unwrap();let record = db.get(id).unwrap();
    assert_eq!(record.value.len(), 100_000);
}

#[test]
fn test_wal_after_compact_persists() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let (id1, id2) = {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<TestRecord>()

            .build()

            .unwrap();let id1 = db.insert(TestRecord {
            value: "record1".to_string(),
        }).unwrap();

        let id2 = db.insert(TestRecord {
            value: "record2".to_string(),
        }).unwrap();

        // Compact
        db.compact().unwrap();

        (id1, id2)
    };

    // Reopen after compact
    let db = Database::open(&db_path)

        .unwrap()

        .register::<TestRecord>()

        .build()

        .unwrap();let record1 = db.get(id1).unwrap();
    let record2 = db.get(id2).unwrap();

    assert_eq!(record1.value, "record1");
    assert_eq!(record2.value, "record2");
}

#[test]
fn test_wal_operations_after_compact() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let db = Database::open(&db_path)


        .unwrap()


        .register::<TestRecord>()


        .build()


        .unwrap();// Insert and compact
    let id1 = db.insert(TestRecord {
        value: "before_compact".to_string(),
    }).unwrap();

    db.compact().unwrap();

    // Insert after compact
    let id2 = db.insert(TestRecord {
        value: "after_compact".to_string(),
    }).unwrap();

    // Update after compact
    db.update(id1, |r: &mut TestRecord| {
        r.value = "updated_after_compact".to_string();
    }).unwrap();

    // Verify
    let record1 = db.get(id1).unwrap();
    let record2 = db.get(id2).unwrap();

    assert_eq!(record1.value, "updated_after_compact");
    assert_eq!(record2.value, "after_compact");
}
