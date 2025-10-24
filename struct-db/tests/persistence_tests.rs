use struct_db::{Database, Table};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tempfile::TempDir;

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
    email: String,
}

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Product {
    name: String,
    price: f64,
}

#[test]
fn test_persistence_insert() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    let id = {
        let db = Database::open(&db_path).unwrap();
        db.register_type::<User>();
        db.insert(user.clone()).unwrap()
    }; // Database drops here

    // Reopen database
    let db = Database::open(&db_path).unwrap();
    db.register_type::<User>();

    let retrieved = db.get_cloned(id).unwrap();
    assert_eq!(retrieved, user);
}

#[test]
fn test_persistence_multiple_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let (id1, id2, id3) = {
        let db = Database::open(&db_path).unwrap();
        db.register_type::<User>();

        let id1 = db.insert(User {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }).unwrap();

        let id2 = db.insert(User {
            name: "Bob".to_string(),
            age: 25,
            email: "bob@example.com".to_string(),
        }).unwrap();

        let id3 = db.insert(User {
            name: "Charlie".to_string(),
            age: 35,
            email: "charlie@example.com".to_string(),
        }).unwrap();

        (id1, id2, id3)
    };

    // Reopen and verify
    let db = Database::open(&db_path).unwrap();
    db.register_type::<User>();

    let user1 = db.get_cloned(id1).unwrap();
    let user2 = db.get_cloned(id2).unwrap();
    let user3 = db.get_cloned(id3).unwrap();

    assert_eq!(user1.name, "Alice");
    assert_eq!(user2.name, "Bob");
    assert_eq!(user3.name, "Charlie");
}

#[test]
fn test_persistence_update() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let id = {
        let db = Database::open(&db_path).unwrap();
        db.register_type::<User>();

        let id = db.insert(User {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }).unwrap();

        db.update(id, |u: &mut User| {
            u.age = 31;
            u.email = "alice.new@example.com".to_string();
        }).unwrap();

        id
    };

    // Reopen and verify update persisted
    let db = Database::open(&db_path).unwrap();
    db.register_type::<User>();

    let user = db.get_cloned(id).unwrap();
    assert_eq!(user.age, 31);
    assert_eq!(user.email, "alice.new@example.com");
}

#[test]
fn test_persistence_delete() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let (id1, id2) = {
        let db = Database::open(&db_path).unwrap();
        db.register_type::<User>();

        let id1 = db.insert(User {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }).unwrap();

        let id2 = db.insert(User {
            name: "Bob".to_string(),
            age: 25,
            email: "bob@example.com".to_string(),
        }).unwrap();

        db.delete(id1).unwrap();

        (id1, id2)
    };

    // Reopen and verify delete persisted
    let db = Database::open(&db_path).unwrap();
    db.register_type::<User>();

    assert!(db.get_cloned(id1).is_err());
    assert!(db.get_cloned(id2).is_ok());
}

#[test]
fn test_persistence_multiple_tables() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let (user_id, product_id) = {
        let db = Database::open(&db_path).unwrap();
        db.register_type::<User>();
        db.register_type::<Product>();

        let user_id = db.insert(User {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }).unwrap();

        let product_id = db.insert(Product {
            name: "Laptop".to_string(),
            price: 999.99,
        }).unwrap();

        (user_id, product_id)
    };

    // Reopen and verify both tables persisted
    let db = Database::open(&db_path).unwrap();
    db.register_type::<User>();
    db.register_type::<Product>();

    let user = db.get_cloned(user_id).unwrap();
    let product = db.get_cloned(product_id).unwrap();

    assert_eq!(user.name, "Alice");
    assert_eq!(product.name, "Laptop");
}

#[test]
fn test_compact_reduces_wal_size() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let db = Database::open(&db_path).unwrap();
    db.register_type::<User>();

    // Insert and delete many records
    for i in 0..100 {
        let id = db.insert(User {
            name: format!("User{}", i),
            age: 20 + i,
            email: format!("user{}@example.com", i),
        }).unwrap();

        // Delete immediately to create unnecessary WAL entries
        if i % 2 == 0 {
            db.delete(id).unwrap();
        }
    }

    let wal_path = db_path.join("wal.log");
    let size_before = std::fs::metadata(&wal_path).unwrap().len();

    // Compact
    db.compact().unwrap();

    let size_after = std::fs::metadata(&wal_path).unwrap().len();

    // WAL should be smaller after compaction
    assert!(size_after < size_before);
}

#[test]
fn test_compact_preserves_data() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let db = Database::open(&db_path).unwrap();
    db.register_type::<User>();

    // Insert data
    let id1 = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }).unwrap();

    let id2 = db.insert(User {
        name: "Bob".to_string(),
        age: 25,
        email: "bob@example.com".to_string(),
    }).unwrap();

    // Update
    db.update(id1, |u: &mut User| {
        u.age = 31;
    }).unwrap();

    // Insert more
    let id3 = db.insert(User {
        name: "Charlie".to_string(),
        age: 35,
        email: "charlie@example.com".to_string(),
    }).unwrap();

    // Compact
    db.compact().unwrap();

    // Verify all data is still accessible
    let user1 = db.get_cloned(id1).unwrap();
    let user2 = db.get_cloned(id2).unwrap();
    let user3 = db.get_cloned(id3).unwrap();

    assert_eq!(user1.age, 31);
    assert_eq!(user2.name, "Bob");
    assert_eq!(user3.name, "Charlie");
}

#[test]
fn test_persistence_after_compact() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let (id1, id2, id3) = {
        let db = Database::open(&db_path).unwrap();
        db.register_type::<User>();

        let id1 = db.insert(User {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }).unwrap();

        let id2 = db.insert(User {
            name: "Bob".to_string(),
            age: 25,
            email: "bob@example.com".to_string(),
        }).unwrap();

        db.update(id1, |u: &mut User| {
            u.age = 31;
        }).unwrap();

        let id3 = db.insert(User {
            name: "Charlie".to_string(),
            age: 35,
            email: "charlie@example.com".to_string(),
        }).unwrap();

        db.compact().unwrap();

        (id1, id2, id3)
    }; // Drop database

    // Reopen and verify compacted data persisted correctly
    let db = Database::open(&db_path).unwrap();
    db.register_type::<User>();

    let user1 = db.get_cloned(id1).unwrap();
    let user2 = db.get_cloned(id2).unwrap();
    let user3 = db.get_cloned(id3).unwrap();

    assert_eq!(user1.age, 31);
    assert_eq!(user2.name, "Bob");
    assert_eq!(user3.name, "Charlie");
}

#[test]
fn test_query_after_reload() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    {
        let db = Database::open(&db_path).unwrap();
        db.register_type::<User>();

        db.insert(User {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }).unwrap();

        db.insert(User {
            name: "Bob".to_string(),
            age: 25,
            email: "bob@example.com".to_string(),
        }).unwrap();

        db.insert(User {
            name: "Charlie".to_string(),
            age: 35,
            email: "charlie@example.com".to_string(),
        }).unwrap();
    }

    // Reopen and query
    let db = Database::open(&db_path).unwrap();
    db.register_type::<User>();

    let results = db
        .query::<User>()
        .filter(|u| u.age >= 30)
        .sort_by(|u| u.age)
        .collect();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1.name, "Alice");
    assert_eq!(results[1].1.name, "Charlie");
}

#[test]
fn test_empty_database_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    {
        let db = Database::open(&db_path).unwrap();
        db.register_type::<User>();
        // Don't insert anything
    }

    // Reopen
    let db = Database::open(&db_path).unwrap();
    db.register_type::<User>();

    let results = db.query::<User>().collect();
    assert_eq!(results.len(), 0);
}
