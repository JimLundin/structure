use struct_db::{Database, Ref, Table};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
}

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Post {
    title: String,
    content: String,
    author: Ref<User>,
}

#[derive(Table, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Comment {
    text: String,
    post: Ref<Post>,
    author: Ref<User>,
}

fn setup_test_db() -> (Database, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db = Database::open(temp_dir.path())
        .unwrap()
        .register::<User>()
        .register::<Post>()
        .register::<Comment>()
        .build()
        .unwrap();
    (db, temp_dir)
}

#[test]
fn test_simple_reference() {
    let (db, _temp_dir) = setup_test_db();

    // Create a user
    let user_id = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
    }).unwrap();

    // Create a post referencing the user
    let post = Post {
        title: "Hello World".to_string(),
        content: "This is my first post".to_string(),
        author: Ref::new(user_id),
    };

    let post_id = db.insert(post.clone()).unwrap();

    // Retrieve post
    let retrieved_post = db.get(post_id).unwrap();

    // Dereference the author
    let author = retrieved_post.author.get(&db).unwrap();
    assert_eq!(author.name, "Alice");
    assert_eq!(author.age, 30);
}

#[test]
fn test_multiple_references_to_same_record() {
    let (db, _temp_dir) = setup_test_db();

    let user_id = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
    }).unwrap();

    let post1_id = db.insert(Post {
        title: "Post 1".to_string(),
        content: "Content 1".to_string(),
        author: Ref::new(user_id),
    }).unwrap();

    let post2_id = db.insert(Post {
        title: "Post 2".to_string(),
        content: "Content 2".to_string(),
        author: Ref::new(user_id),
    }).unwrap();

    let post1 = db.get(post1_id).unwrap();
    let post2 = db.get(post2_id).unwrap();

    let author1 = post1.author.get(&db).unwrap();
    let author2 = post2.author.get(&db).unwrap();

    assert_eq!(author1.name, "Alice");
    assert_eq!(author2.name, "Alice");
}

#[test]
fn test_nested_references() {
    let (db, _temp_dir) = setup_test_db();

    // Create user
    let user_id = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
    }).unwrap();

    // Create post
    let post_id = db.insert(Post {
        title: "Hello World".to_string(),
        content: "This is my first post".to_string(),
        author: Ref::new(user_id),
    }).unwrap();

    // Create comment
    let comment = Comment {
        text: "Great post!".to_string(),
        post: Ref::new(post_id),
        author: Ref::new(user_id),
    };

    let comment_id = db.insert(comment).unwrap();

    // Retrieve and traverse references
    let retrieved_comment = db.get(comment_id).unwrap();
    let post = retrieved_comment.post.get(&db).unwrap();
    let post_author = post.author.get(&db).unwrap();
    let comment_author = retrieved_comment.author.get(&db).unwrap();

    assert_eq!(retrieved_comment.text, "Great post!");
    assert_eq!(post.title, "Hello World");
    assert_eq!(post_author.name, "Alice");
    assert_eq!(comment_author.name, "Alice");
}

#[test]
fn test_reference_to_deleted_record() {
    let (db, _temp_dir) = setup_test_db();

    let user_id = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
    }).unwrap();

    let post_id = db.insert(Post {
        title: "Hello World".to_string(),
        content: "This is my first post".to_string(),
        author: Ref::new(user_id),
    }).unwrap();

    // Delete the user
    db.delete(user_id).unwrap();

    // Try to dereference
    let post = db.get(post_id).unwrap();
    let result = post.author.get(&db);

    // Should fail because the user no longer exists
    assert!(result.is_err());
}

#[test]
fn test_reference_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let (user_id, post_id) = {
        let db = Database::open(&db_path)

            .unwrap()

            .register::<User>()

            .register::<Post>()

            .build()

            .unwrap();let user_id = db.insert(User {
            name: "Alice".to_string(),
            age: 30,
        }).unwrap();

        let post_id = db.insert(Post {
            title: "Hello World".to_string(),
            content: "This is my first post".to_string(),
            author: Ref::new(user_id),
        }).unwrap();

        (user_id, post_id)
    };

    // Reopen database
    let db = Database::open(&db_path)

        .unwrap()

        .register::<User>()

        .register::<Post>()

        .build()

        .unwrap();let post = db.get(post_id).unwrap();
    let author = post.author.get(&db).unwrap();

    assert_eq!(author.name, "Alice");
    assert_eq!(post.title, "Hello World");
}

#[test]
fn test_query_with_references() {
    let (db, _temp_dir) = setup_test_db();

    let alice_id = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
    }).unwrap();

    let bob_id = db.insert(User {
        name: "Bob".to_string(),
        age: 25,
    }).unwrap();

    db.insert(Post {
        title: "Alice's Post 1".to_string(),
        content: "Content 1".to_string(),
        author: Ref::new(alice_id),
    }).unwrap();

    db.insert(Post {
        title: "Alice's Post 2".to_string(),
        content: "Content 2".to_string(),
        author: Ref::new(alice_id),
    }).unwrap();

    db.insert(Post {
        title: "Bob's Post".to_string(),
        content: "Content 3".to_string(),
        author: Ref::new(bob_id),
    }).unwrap();

    // Query posts by Alice
    let alice_posts = db
        .query::<Post>().unwrap()
        .filter(move |p| p.author.id() == alice_id)
        .collect();

    assert_eq!(alice_posts.len(), 2);

    for (_id, post) in alice_posts {
        let author = post.author.get(&db).unwrap();
        assert_eq!(author.name, "Alice");
    }
}

#[test]
fn test_update_referenced_record() {
    let (db, _temp_dir) = setup_test_db();

    let user_id = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
    }).unwrap();

    let post_id = db.insert(Post {
        title: "Hello World".to_string(),
        content: "This is my first post".to_string(),
        author: Ref::new(user_id),
    }).unwrap();

    // Update the user
    db.update(user_id, |u: &mut User| {
        u.age = 31;
        u.name = "Alice Smith".to_string();
    }).unwrap();

    // Retrieve post and check author
    let post = db.get(post_id).unwrap();
    let author = post.author.get(&db).unwrap();

    assert_eq!(author.name, "Alice Smith");
    assert_eq!(author.age, 31);
}

#[test]
fn test_reference_id_method() {
    let (db, _temp_dir) = setup_test_db();

    let user_id = db.insert(User {
        name: "Alice".to_string(),
        age: 30,
    }).unwrap();

    let post = Post {
        title: "Hello World".to_string(),
        content: "This is my first post".to_string(),
        author: Ref::new(user_id),
    };

    // Test that we can get the ID without dereferencing
    assert_eq!(post.author.id(), user_id);
}

#[test]
fn test_circular_references() {
    // This test demonstrates that circular references can be stored
    // (though dereferencing them infinitely would be a logic error)

    #[derive(Table, Serialize, Deserialize, Clone, Debug)]
    struct Node {
        name: String,
        next: Option<Ref<Node>>,
    }

    let temp_dir = TempDir::new().unwrap();
    let db = Database::open(temp_dir.path()).unwrap();
    db.register::<Node>();

    // Create node without reference first
    let node1_id = db.insert(Node {
        name: "Node1".to_string(),
        next: None,
    }).unwrap();

    let node2_id = db.insert(Node {
        name: "Node2".to_string(),
        next: Some(Ref::new(node1_id)),
    }).unwrap();

    // Update node1 to reference node2 (creating a cycle)
    db.update(node1_id, |n: &mut Node| {
        n.next = Some(Ref::new(node2_id));
    }).unwrap();

    // Verify the cycle exists
    let node1 = db.get(node1_id).unwrap();
    let node2 = node1.next.unwrap().get(&db).unwrap();
    let node1_again = node2.next.unwrap().get(&db).unwrap();

    assert_eq!(node1.name, "Node1");
    assert_eq!(node2.name, "Node2");
    assert_eq!(node1_again.name, "Node1");
}
