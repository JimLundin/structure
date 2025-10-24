use struct_db::{Database, Table};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Counter {
    value: u32,
}

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
}

#[test]
fn test_concurrent_reads() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(Database::open(temp_dir.path()).unwrap());
    db.register_type::<User>();

    // Insert test data
    let id = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
    }).unwrap();

    // Spawn multiple reader threads
    let mut handles = vec![];

    for i in 0..10 {
        let db_clone = Arc::clone(&db);
        let id_clone = id;

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let user = db_clone.get_cloned(id_clone).unwrap();
                assert_eq!(user.name, "Alice");
                assert_eq!(user.age, 30);
            }
            i
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_inserts() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(Database::open(temp_dir.path()).unwrap());
    db.register_type::<User>();

    let mut handles = vec![];

    // Spawn multiple threads inserting data
    for i in 0..10 {
        let db_clone = Arc::clone(&db);

        let handle = thread::spawn(move || {
            let mut ids = vec![];
            for j in 0..10 {
                let id = db_clone.insert(User {
                    name: format!("User{}-{}", i, j),
                    age: 20 + i,
                }).unwrap();
                ids.push(id);
            }
            ids
        });

        handles.push(handle);
    }

    // Collect all IDs
    let mut all_ids = vec![];
    for handle in handles {
        let ids = handle.join().unwrap();
        all_ids.extend(ids);
    }

    // Verify all inserts succeeded and IDs are unique
    assert_eq!(all_ids.len(), 100);

    let mut id_set = std::collections::HashSet::new();
    for id in &all_ids {
        assert!(id_set.insert(id.as_u64()));
    }

    // Verify all records can be retrieved
    for id in all_ids {
        assert!(db.get_cloned(id).is_ok());
    }
}

#[test]
fn test_concurrent_updates() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(Database::open(temp_dir.path()).unwrap());
    db.register_type::<Counter>();

    // Insert a counter
    let id = db.insert(Counter { value: 0 }).unwrap();

    let mut handles = vec![];

    // Spawn multiple threads updating the counter
    for _ in 0..10 {
        let db_clone = Arc::clone(&db);
        let id_clone = id;

        let handle = thread::spawn(move || {
            for _ in 0..10 {
                db_clone.update(id_clone, |c: &mut Counter| {
                    c.value += 1;
                }).unwrap();
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final value
    let counter = db.get_cloned(id).unwrap();
    assert_eq!(counter.value, 100);
}

#[test]
fn test_concurrent_mixed_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(Database::open(temp_dir.path()).unwrap());
    db.register_type::<User>();

    // Insert some initial data
    let id = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
    }).unwrap();

    let mut handles = vec![];

    // Reader threads
    for _ in 0..5 {
        let db_clone = Arc::clone(&db);
        let id_clone = id;

        let handle = thread::spawn(move || {
            for _ in 0..50 {
                let _ = db_clone.get_cloned(id_clone);
            }
        });

        handles.push(handle);
    }

    // Writer threads (updates)
    for i in 0..3 {
        let db_clone = Arc::clone(&db);
        let id_clone = id;

        let handle = thread::spawn(move || {
            for j in 0..20 {
                db_clone.update(id_clone, |u: &mut User| {
                    u.age = 30 + (i * 20 + j) as u32;
                }).unwrap();
            }
        });

        handles.push(handle);
    }

    // Writer threads (inserts)
    for i in 0..2 {
        let db_clone = Arc::clone(&db);

        let handle = thread::spawn(move || {
            for j in 0..10 {
                db_clone.insert(User {
                    name: format!("User{}-{}", i, j),
                    age: 25,
                }).unwrap();
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify database is still functional
    let user = db.get_cloned(id).unwrap();
    assert_eq!(user.name, "Alice");

    let all_users = db.query::<User>().collect();
    assert_eq!(all_users.len(), 21); // 1 original + 20 inserted
}

#[test]
fn test_concurrent_queries() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(Database::open(temp_dir.path()).unwrap());
    db.register_type::<User>();

    // Insert test data
    for i in 0..50 {
        db.insert(User {
            name: format!("User{}", i),
            age: 20 + (i % 30),
        }).unwrap();
    }

    let mut handles = vec![];

    // Spawn multiple threads querying
    for _ in 0..10 {
        let db_clone = Arc::clone(&db);

        let handle = thread::spawn(move || {
            for _ in 0..20 {
                let results = db_clone
                    .query::<User>()
                    .filter(|u| u.age > 30)
                    .collect();

                assert!(results.len() > 0);
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_delete_and_read() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(Database::open(temp_dir.path()).unwrap());
    db.register_type::<User>();

    // Insert many records
    let mut ids = vec![];
    for i in 0..100 {
        let id = db.insert(User {
            name: format!("User{}", i),
            age: 20 + i,
        }).unwrap();
        ids.push(id);
    }

    let ids = Arc::new(ids);
    let mut handles = vec![];

    // Reader threads
    for _ in 0..5 {
        let db_clone = Arc::clone(&db);
        let ids_clone = Arc::clone(&ids);

        let handle = thread::spawn(move || {
            for _ in 0..50 {
                for id in ids_clone.iter() {
                    let _ = db_clone.get_cloned(*id);
                }
            }
        });

        handles.push(handle);
    }

    // Deleter thread
    let db_clone = Arc::clone(&db);
    let ids_clone = Arc::clone(&ids);

    let delete_handle = thread::spawn(move || {
        for id in ids_clone.iter().take(50) {
            db_clone.delete(*id).unwrap();
        }
    });

    handles.push(delete_handle);

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify first 50 are deleted, last 50 still exist
    for id in ids.iter().take(50) {
        assert!(db.get_cloned(*id).is_err());
    }

    for id in ids.iter().skip(50) {
        assert!(db.get_cloned(*id).is_ok());
    }
}

#[test]
fn test_concurrent_query_and_insert() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(Database::open(temp_dir.path()).unwrap());
    db.register_type::<User>();

    // Insert initial data
    for i in 0..20 {
        db.insert(User {
            name: format!("User{}", i),
            age: 20 + i,
        }).unwrap();
    }

    let mut handles = vec![];

    // Query threads
    for _ in 0..5 {
        let db_clone = Arc::clone(&db);

        let handle = thread::spawn(move || {
            for _ in 0..30 {
                let results = db_clone.query::<User>().collect();
                assert!(results.len() >= 20); // At least the initial records
            }
        });

        handles.push(handle);
    }

    // Insert threads
    for i in 0..5 {
        let db_clone = Arc::clone(&db);

        let handle = thread::spawn(move || {
            for j in 0..10 {
                db_clone.insert(User {
                    name: format!("NewUser{}-{}", i, j),
                    age: 50 + j,
                }).unwrap();
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final count
    let all_users = db.query::<User>().collect();
    assert_eq!(all_users.len(), 70); // 20 initial + 50 new
}
