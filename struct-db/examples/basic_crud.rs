use serde::{Deserialize, Serialize};
use struct_db::{Database, Table};

/// A simple User struct demonstrating basic CRUD operations
#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct User {
    name: String,
    email: String,
    age: u32,
}

fn main() -> anyhow::Result<()> {
    println!("=== Basic CRUD Operations Example ===\n");

    // 1. Open database
    println!("1. Opening database...");
    let db = Database::open("./data/basic_crud")?
        .register:<User>()
        .build()?;
    println!("   Database opened successfully\n");

    // 2. CREATE - Insert users
    println!("2. Creating users...");
    let alice_id = db.insert(User {
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    })?;
    println!("   Created user: {} (ID: {})", "Alice Johnson", alice_id);

    let bob_id = db.insert(User {
        name: "Bob Smith".to_string(),
        email: "bob@example.com".to_string(),
        age: 25,
    })?;
    println!("   Created user: {} (ID: {})", "Bob Smith", bob_id);

    let charlie_id = db.insert(User {
        name: "Charlie Brown".to_string(),
        email: "charlie@example.com".to_string(),
        age: 35,
    })?;
    println!("   Created user: {} (ID: {})\n", "Charlie Brown", charlie_id);

    // 3. READ - Retrieve a single user
    println!("3. Reading user by ID...");
    let user = db.get_cloned(alice_id)?;
    println!("   Found: {} ({}, age {})\n", user.name, user.email, user.age);

    // 4. READ - Query all users
    println!("4. Querying all users...");
    let all_users = db.query::<User>()?.collect();
    println!("   Total users: {}", all_users.len());
    for (id, user) in &all_users {
        println!("     - {}: {} ({}, age {})", id, user.name, user.email, user.age);
    }
    println!();

    // 5. READ - Query with filters
    println!("5. Querying users over 30...");
    let older_users = db
        .query::<User>()
        .filter(|u| u.age > 30)
        .collect();
    println!("   Found {} users:", older_users.len());
    for (id, user) in &older_users {
        println!("     - {}: {} (age {})", id, user.name, user.age);
    }
    println!();

    // 6. READ - Query with sorting
    println!("6. Querying users sorted by name...");
    let sorted_users = db
        .query::<User>()
        .sort_by(|u| u.name.clone())
        .collect();
    for (id, user) in &sorted_users {
        println!("     - {}: {}", id, user.name);
    }
    println!();

    // 7. UPDATE - Modify a user
    println!("7. Updating user...");
    println!("   Before: {}", db.get_cloned(bob_id)?email);
    db.update(bob_id, |user| {
        user.email = "robert.smith@example.com".to_string();
        user.age = 26;
    })?;
    let updated_user = db.get_cloned(bob_id)?;
    println!("   After: {} (age {})\n", updated_user.email, updated_user.age);

    // 8. DELETE - Remove a user
    println!("8. Deleting user...");
    println!("   Users before delete: {}", db.query::<User>()?.count());
    db.delete(charlie_id)?;
    println!("   Users after delete: {}", db.query::<User>()?.count());
    println!("   Deleted user: {}\n", charlie_id);

    // 9. Verify deletion
    println!("9. Verifying deletion...");
    let remaining = db.query::<User>()?.collect();
    println!("   Remaining users:");
    for (id, user) in &remaining {
        println!("     - {}: {}", id, user.name);
    }
    println!();

    // 10. Demonstrate error handling
    println!("10. Error handling...");
    match db.get_cloned(charlie_id) {
        Ok(user) => println!("   Unexpected: Found deleted user {}", user.name),
        Err(e) => println!("   Expected error: {}", e),
    }
    println!();

    // 11. Advanced queries
    println!("11. Advanced queries...");

    // Count
    let count = db.query::<User>()?.count();
    println!("   Total users: {}", count);

    // First match
    if let Some((id, user)) = db.query::<User>()?
        .filter(|u| u.age < 30)
        .first()
    {
        println!("   First user under 30: {} (ID: {})", user.name, id);
    }

    // Chained filters
    let filtered = db.query::<User>()?
        .filter(|u| u.age >= 25)
        .filter(|u| u.name.starts_with("A"))
        .collect();
    println!("   Users aged 25+ with name starting with 'A': {}", filtered.len());

    // Limit results
    let limited = db.query::<User>()?
        .limit(1)
        .collect();
    println!("   Limited to 1 user: {}", limited.len());
    println!();

    // 12. Persistence demonstration
    println!("12. Demonstrating persistence...");
    println!("   Closing database...");
    drop(db);

    println!("   Reopening database...");
    let db = Database::open("./data/basic_crud")?
        .register:<User>()
        .build()?;

    let reloaded_users = db.query::<User>()??collect();
    println!("   Reloaded {} users after restart:", reloaded_users.len());
    for (id, user) in &reloaded_users {
        println!("     - {}: {}", id, user.name);
    }
    println!();

    println!("=== Example completed successfully! ===");

    Ok(())
}
