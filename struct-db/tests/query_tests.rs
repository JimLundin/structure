use struct_db::{Database, Table};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
    email: String,
    active: bool,
}

fn setup_test_db_with_users() -> (Database, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db = Database::open(temp_dir.path()).unwrap();
    db.register_type::<User>();

    // Insert test data
    let users = vec![
        User {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
            active: true,
        },
        User {
            name: "Bob".to_string(),
            age: 25,
            email: "bob@example.com".to_string(),
            active: false,
        },
        User {
            name: "Charlie".to_string(),
            age: 35,
            email: "charlie@example.com".to_string(),
            active: true,
        },
        User {
            name: "Diana".to_string(),
            age: 28,
            email: "diana@example.com".to_string(),
            active: true,
        },
        User {
            name: "Eve".to_string(),
            age: 32,
            email: "eve@example.com".to_string(),
            active: false,
        },
    ];

    for user in users {
        db.insert(user).unwrap();
    }

    (db, temp_dir)
}

#[test]
fn test_query_all() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db.query::<User>().collect();

    assert_eq!(results.len(), 5);
}

#[test]
fn test_query_filter_single() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .filter(|u| u.age > 30)
        .collect();

    assert_eq!(results.len(), 2); // Charlie (35) and Eve (32)

    for (_id, user) in results {
        assert!(user.age > 30);
    }
}

#[test]
fn test_query_filter_active() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .filter(|u| u.active)
        .collect();

    assert_eq!(results.len(), 3); // Alice, Charlie, Diana

    for (_id, user) in results {
        assert!(user.active);
    }
}

#[test]
fn test_query_filter_chained() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .filter(|u| u.active)
        .filter(|u| u.age >= 30)
        .collect();

    assert_eq!(results.len(), 2); // Alice (30) and Charlie (35)

    for (_id, user) in results {
        assert!(user.active);
        assert!(user.age >= 30);
    }
}

#[test]
fn test_query_sort_by_age() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .sort_by(|u| u.age)
        .collect();

    assert_eq!(results.len(), 5);

    // Verify sorting
    let ages: Vec<u32> = results.iter().map(|(_, u)| u.age).collect();
    assert_eq!(ages, vec![25, 28, 30, 32, 35]);
}

#[test]
fn test_query_sort_by_name() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .sort_by(|u| u.name.clone())
        .collect();

    assert_eq!(results.len(), 5);

    // Verify sorting
    let names: Vec<String> = results.iter().map(|(_, u)| u.name.clone()).collect();
    assert_eq!(names, vec!["Alice", "Bob", "Charlie", "Diana", "Eve"]);
}

#[test]
fn test_query_sort_by_cmp() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .sort_by_cmp(|a, b| b.age.cmp(&a.age)) // Descending order
        .collect();

    assert_eq!(results.len(), 5);

    // Verify sorting (descending)
    let ages: Vec<u32> = results.iter().map(|(_, u)| u.age).collect();
    assert_eq!(ages, vec![35, 32, 30, 28, 25]);
}

#[test]
fn test_query_limit() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .limit(3)
        .collect();

    assert_eq!(results.len(), 3);
}

#[test]
fn test_query_filter_sort_limit() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .filter(|u| u.active)
        .sort_by(|u| u.age)
        .limit(2)
        .collect();

    assert_eq!(results.len(), 2);

    // Should be Diana (28) and Alice (30)
    let ages: Vec<u32> = results.iter().map(|(_, u)| u.age).collect();
    assert_eq!(ages, vec![28, 30]);

    for (_id, user) in results {
        assert!(user.active);
    }
}

#[test]
fn test_query_first() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let result = db
        .query::<User>()
        .filter(|u| u.name == "Charlie")
        .first();

    assert!(result.is_some());
    let (_id, user) = result.unwrap();
    assert_eq!(user.name, "Charlie");
}

#[test]
fn test_query_first_not_found() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let result = db
        .query::<User>()
        .filter(|u| u.name == "Nonexistent")
        .first();

    assert!(result.is_none());
}

#[test]
fn test_query_count() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let count = db
        .query::<User>()
        .filter(|u| u.active)
        .count();

    assert_eq!(count, 3);
}

#[test]
fn test_query_count_all() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let count = db.query::<User>().count();

    assert_eq!(count, 5);
}

#[test]
fn test_query_empty_result() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .filter(|u| u.age > 100)
        .collect();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_query_complex_filter() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .filter(|u| u.active && u.age >= 30 && u.name.starts_with('C'))
        .collect();

    assert_eq!(results.len(), 1);

    let (_id, user) = &results[0];
    assert_eq!(user.name, "Charlie");
}

#[test]
fn test_query_with_limit_zero() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .limit(0)
        .collect();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_query_limit_larger_than_results() {
    let (db, _temp_dir) = setup_test_db_with_users();

    let results = db
        .query::<User>()
        .limit(100)
        .collect();

    assert_eq!(results.len(), 5);
}
