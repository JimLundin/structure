use struct_db::{Database, Id, Table};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
    email: String,
    active: bool,
}

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Product {
    name: String,
    price: f64,
    in_stock: bool,
}

fn setup_test_db() -> (Database, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db = Database::open(temp_dir.path())
        .unwrap()
        .register::<User>()
        .register::<Product>()
        .build()
        .unwrap();
    (db, temp_dir)
}

#[test]
fn test_insert_and_get() {
    let (db, _temp_dir) = setup_test_db();

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
        active: true,
    };

    let id = db.insert(user.clone()).unwrap();
    let retrieved = db.get(id).unwrap();

    assert_eq!(retrieved, user);
}

#[test]
fn test_insert_multiple_records() {
    let (db, _temp_dir) = setup_test_db();

    let user1 = User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
        active: true,
    };

    let user2 = User {
        name: "Bob".to_string(),
        age: 25,
        email: "bob@example.com".to_string(),
        active: false,
    };

    let id1 = db.insert(user1.clone()).unwrap();
    let id2 = db.insert(user2.clone()).unwrap();

    assert_ne!(id1, id2);

    let retrieved1 = db.get(id1).unwrap();
    let retrieved2 = db.get(id2).unwrap();

    assert_eq!(retrieved1, user1);
    assert_eq!(retrieved2, user2);
}

#[test]
fn test_update_record() {
    let (db, _temp_dir) = setup_test_db();

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
        active: true,
    };

    let id = db.insert(user).unwrap();

    db.update(id, |u: &mut User| {
        u.age = 31;
        u.email = "alice.new@example.com".to_string();
    }).unwrap();

    let updated = db.get(id).unwrap();
    assert_eq!(updated.age, 31);
    assert_eq!(updated.email, "alice.new@example.com");
    assert_eq!(updated.name, "Alice");
}

#[test]
fn test_delete_record() {
    let (db, _temp_dir) = setup_test_db();

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
        active: true,
    };

    let id = db.insert(user).unwrap();

    // Verify it exists
    assert!(db.get(id).is_ok());

    // Delete it
    db.delete(id).unwrap();

    // Verify it's gone
    assert!(db.get(id).is_err());
}

#[test]
fn test_get_nonexistent_record() {
    let (db, _temp_dir) = setup_test_db();

    let fake_id: Id<User> = Id::from(999);
    let result = db.get(fake_id);

    assert!(result.is_err());
}

#[test]
fn test_update_nonexistent_record() {
    let (db, _temp_dir) = setup_test_db();

    let fake_id: Id<User> = Id::from(999);
    let result = db.update(fake_id, |u: &mut User| {
        u.age = 100;
    });

    assert!(result.is_err());
}

#[test]
fn test_delete_nonexistent_record() {
    let (db, _temp_dir) = setup_test_db();

    let fake_id: Id<User> = Id::from(999);
    let result = db.delete(fake_id);

    assert!(result.is_err());
}

#[test]
fn test_multiple_tables() {
    let (db, _temp_dir) = setup_test_db();

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
        active: true,
    };

    let product = Product {
        name: "Laptop".to_string(),
        price: 999.99,
        in_stock: true,
    };

    let user_id = db.insert(user.clone()).unwrap();
    let product_id = db.insert(product.clone()).unwrap();

    let retrieved_user = db.get(user_id).unwrap();
    let retrieved_product = db.get(product_id).unwrap();

    assert_eq!(retrieved_user, user);
    assert_eq!(retrieved_product, product);
}

#[test]
fn test_insert_returns_sequential_ids() {
    let (db, _temp_dir) = setup_test_db();

    let user1 = User {
        name: "User1".to_string(),
        age: 20,
        email: "user1@example.com".to_string(),
        active: true,
    };

    let user2 = User {
        name: "User2".to_string(),
        age: 21,
        email: "user2@example.com".to_string(),
        active: true,
    };

    let user3 = User {
        name: "User3".to_string(),
        age: 22,
        email: "user3@example.com".to_string(),
        active: true,
    };

    let id1 = db.insert(user1).unwrap();
    let id2 = db.insert(user2).unwrap();
    let id3 = db.insert(user3).unwrap();

    // IDs should be sequential
    assert_eq!(id1.as_u64() + 1, id2.as_u64());
    assert_eq!(id2.as_u64() + 1, id3.as_u64());
}
