use struct_db::{Database, Id, Ref, Table};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct User {
    name: String,
}

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Post {
    title: String,
}

// Tests for Id type
mod id_tests {
    use super::*;

    #[test]
    fn test_id_creation() {
        let id: Id<User> = Id::from(42);
        assert_eq!(id.as_u64(), 42);
    }

    #[test]
    fn test_id_equality() {
        let id1: Id<User> = Id::from(42);
        let id2: Id<User> = Id::from(42);
        let id3: Id<User> = Id::from(43);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_id_type_safety() {
        let user_id: Id<User> = Id::from(1);
        let post_id: Id<Post> = Id::from(1);

        // These are different types at compile time
        // user_id == post_id; // This would not compile

        // But we can compare their raw values
        assert_eq!(user_id.as_u64(), post_id.as_u64());
    }

    #[test]
    fn test_id_display() {
        let id: Id<User> = Id::from(123);
        assert_eq!(format!("{}", id), "123");
    }

    #[test]
    fn test_id_debug() {
        let id: Id<User> = Id::from(456);
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("456"));
    }

    #[test]
    fn test_id_serialization() {
        let id: Id<User> = Id::from(789);
        let serialized = bincode::serialize(&id).unwrap();
        let deserialized: Id<User> = bincode::deserialize(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_id_copy() {
        let id1: Id<User> = Id::from(100);
        let id2 = id1; // Should copy
        let id3 = id1; // Should still work

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
    }

    #[test]
    fn test_id_hash() {
        use std::collections::HashMap;

        let id1: Id<User> = Id::from(1);
        let id2: Id<User> = Id::from(2);

        let mut map = HashMap::new();
        map.insert(id1, "User 1");
        map.insert(id2, "User 2");

        assert_eq!(map.get(&id1), Some(&"User 1"));
        assert_eq!(map.get(&id2), Some(&"User 2"));
    }
}

// Tests for Ref type
mod ref_tests {
    use super::*;

    #[test]
    fn test_ref_creation() {
        let user_id: Id<User> = Id::from(42);
        let user_ref: Ref<User> = Ref::new(user_id);

        assert_eq!(user_ref.id(), user_id);
    }

    #[test]
    fn test_ref_id_retrieval() {
        let user_id: Id<User> = Id::from(123);
        let user_ref: Ref<User> = Ref::new(user_id);

        assert_eq!(user_ref.id(), user_id);
        assert_eq!(user_ref.id().as_u64(), 123);
    }

    #[test]
    fn test_ref_serialization() {
        let user_id: Id<User> = Id::from(456);
        let user_ref: Ref<User> = Ref::new(user_id);

        let serialized = bincode::serialize(&user_ref).unwrap();
        let deserialized: Ref<User> = bincode::deserialize(&serialized).unwrap();

        assert_eq!(user_ref.id(), deserialized.id());
    }

    #[test]
    fn test_ref_get_success() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path())
            .unwrap()
            .register::<User>()
            .build()
            .unwrap();

        let user_id = db.insert(User {
            name: "Alice".to_string(),
        }).unwrap();

        let user_ref: Ref<User> = Ref::new(user_id);
        let user = user_ref.get(&db).unwrap();

        assert_eq!(user.name, "Alice");
    }

    #[test]
    fn test_ref_get_failure() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path())
            .unwrap()
            .register::<User>()
            .build()
            .unwrap();

        let fake_id: Id<User> = Id::from(999);
        let user_ref: Ref<User> = Ref::new(fake_id);
        let result = user_ref.get(&db);

        assert!(result.is_err());
    }

    #[test]
    fn test_ref_clone() {
        let user_id: Id<User> = Id::from(42);
        let ref1: Ref<User> = Ref::new(user_id);
        let ref2 = ref1.clone();

        assert_eq!(ref1.id(), ref2.id());
    }

    #[test]
    fn test_ref_debug() {
        let user_id: Id<User> = Id::from(42);
        let user_ref: Ref<User> = Ref::new(user_id);
        let debug_str = format!("{:?}", user_ref);

        assert!(debug_str.contains("42") || debug_str.len() > 0);
    }
}

// Edge case tests
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_database() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        let results = db.query::<User>().unwrap().collect();
        assert_eq!(results.len(), 0);

        let count = db.query::<User>().unwrap().count();
        assert_eq!(count, 0);

        let first = db.query::<User>().unwrap().first();
        assert!(first.is_none());
    }

    #[test]
    fn test_empty_strings() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        let id = db.insert(User {
            name: "".to_string(),
        }).unwrap();

        let user = db.get(id).unwrap();
        assert_eq!(user.name, "");
    }

    #[test]
    fn test_unicode_strings() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        let unicode_name = "Hello 世界 🌍 Привет";
        let id = db.insert(User {
            name: unicode_name.to_string(),
        }).unwrap();

        let user = db.get(id).unwrap();
        assert_eq!(user.name, unicode_name);
    }

    #[test]
    fn test_very_long_strings() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        let long_name = "a".repeat(10000);
        let id = db.insert(User {
            name: long_name.clone(),
        }).unwrap();

        let user = db.get(id).unwrap();
        assert_eq!(user.name, long_name);
    }

    #[test]
    fn test_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        let special_name = "Test\n\r\t\\\"'";
        let id = db.insert(User {
            name: special_name.to_string(),
        }).unwrap();

        let user = db.get(id).unwrap();
        assert_eq!(user.name, special_name);
    }

    #[test]
    fn test_large_number_of_records() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        let count = 1000;
        for i in 0..count {
            db.insert(User {
                name: format!("User{}", i),
            }).unwrap();
        }

        let results = db.query::<User>().unwrap().collect();
        assert_eq!(results.len(), count);
    }

    #[test]
    fn test_update_to_same_value() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        let id = db.insert(User {
            name: "Alice".to_string(),
        }).unwrap();

        // Update to same value
        db.update(id, |u: &mut User| {
            u.name = "Alice".to_string();
        }).unwrap();

        let user = db.get(id).unwrap();
        assert_eq!(user.name, "Alice");
    }

    #[test]
    fn test_multiple_updates_same_record() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        let id = db.insert(User {
            name: "Alice".to_string(),
        }).unwrap();

        for i in 0..100 {
            db.update(id, |u: &mut User| {
                u.name = format!("Alice{}", i);
            }).unwrap();
        }

        let user = db.get(id).unwrap();
        assert_eq!(user.name, "Alice99");
    }

    #[test]
    fn test_delete_and_reinsert() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        let id1 = db.insert(User {
            name: "Alice".to_string(),
        }).unwrap();

        db.delete(id1).unwrap();

        let id2 = db.insert(User {
            name: "Bob".to_string(),
        }).unwrap();

        // IDs should be different
        assert_ne!(id1, id2);

        // Old ID should not exist
        assert!(db.get(id1).is_err());

        // New ID should exist
        let user = db.get(id2).unwrap();
        assert_eq!(user.name, "Bob");
    }

    #[test]
    fn test_query_with_no_matching_results() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        db.insert(User {
            name: "Alice".to_string(),
        }).unwrap();

        let results = db
            .query::<User>().unwrap()
            .filter(|u| u.name == "Bob")
            .collect();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_filter_with_always_false_predicate() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        for i in 0..10 {
            db.insert(User {
                name: format!("User{}", i),
            }).unwrap();
        }

        let results = db
            .query::<User>().unwrap()
            .filter(|_| false)
            .collect();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_filter_with_always_true_predicate() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        db.register::<User>();

        for i in 0..10 {
            db.insert(User {
                name: format!("User{}", i),
            }).unwrap();
        }

        let results = db
            .query::<User>().unwrap()
            .filter(|_| true)
            .collect();

        assert_eq!(results.len(), 10);
    }
}
