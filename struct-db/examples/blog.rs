use serde::{Deserialize, Serialize};
use struct_db::{Database, Id, Ref, Table};

/// Blog author
#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct Author {
    username: String,
    full_name: String,
    bio: String,
}

/// Blog post
#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct Post {
    title: String,
    content: String,
    author: Ref<Author>,
    published: bool,
    views: u64,
    created_at: u64,  // Unix timestamp
}

/// Comment on a post
#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct Comment {
    post: Ref<Post>,
    author: Ref<Author>,
    content: String,
    created_at: u64,
}

/// Tag for categorizing posts
#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct Tag {
    name: String,
}

/// Association between posts and tags (many-to-many)
#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct PostTag {
    post: Ref<Post>,
    tag: Ref<Tag>,
}

fn main() -> anyhow::Result<()> {
    println!("=== Blog System Example ===\n");

    // Setup database
    let db = Database::open("./data/blog")?;
    db.register:::<Author>();
    db.register:::<Post>();
    db.register:::<Comment>();
    db.register:::<Tag>();
    db.register:::<PostTag>();

    println!("1. Creating authors...");
    let alice = db.insert(Author {
        username: "alice_dev".to_string(),
        full_name: "Alice Developer".to_string(),
        bio: "Full-stack developer and tech blogger".to_string(),
    })?;
    println!("   Created author: @alice_dev");

    let bob = db.insert(Author {
        username: "bob_rust".to_string(),
        full_name: "Bob Rustacean".to_string(),
        bio: "Rust enthusiast and systems programmer".to_string(),
    })?;
    println!("   Created author: @bob_rust");

    let carol = db.insert(Author {
        username: "carol_writes".to_string(),
        full_name: "Carol Writer".to_string(),
        bio: "Technical writer and documentation specialist".to_string(),
    })?;
    println!("   Created author: @carol_writes\n");

    println!("2. Creating tags...");
    let rust_tag = db.insert(Tag {
        name: "Rust".to_string(),
    })?;
    let tutorial_tag = db.insert(Tag {
        name: "Tutorial".to_string(),
    })?;
    let webdev_tag = db.insert(Tag {
        name: "WebDev".to_string(),
    })?;
    let database_tag = db.insert(Tag {
        name: "Database".to_string(),
    })?;
    println!("   Created tags: Rust, Tutorial, WebDev, Database\n");

    println!("3. Creating blog posts...");

    let post1 = db.insert(Post {
        title: "Getting Started with Rust".to_string(),
        content: "Rust is a systems programming language...".to_string(),
        author: Ref::new(alice),
        published: true,
        views: 1250,
        created_at: 1698765000,
    })?;
    println!("   Published: 'Getting Started with Rust'");

    // Tag post1
    db.insert(PostTag { post: Ref::new(post1), tag: Ref::new(rust_tag) })?;
    db.insert(PostTag { post: Ref::new(post1), tag: Ref::new(tutorial_tag) })?;

    let post2 = db.insert(Post {
        title: "Building a Web Server in Rust".to_string(),
        content: "Let's build a simple web server...".to_string(),
        author: Ref::new(bob),
        published: true,
        views: 3420,
        created_at: 1698766000,
    })?;
    println!("   Published: 'Building a Web Server in Rust'");

    db.insert(PostTag { post: Ref::new(post2), tag: Ref::new(rust_tag) })?;
    db.insert(PostTag { post: Ref::new(post2), tag: Ref::new(webdev_tag) })?;
    db.insert(PostTag { post: Ref::new(post2), tag: Ref::new(tutorial_tag) })?;

    let post3 = db.insert(Post {
        title: "Understanding Database Design".to_string(),
        content: "Database design is crucial...".to_string(),
        author: Ref::new(alice),
        published: true,
        views: 890,
        created_at: 1698767000,
    })?;
    println!("   Published: 'Understanding Database Design'");

    db.insert(PostTag { post: Ref::new(post3), tag: Ref::new(database_tag) })?;

    let _post4 = db.insert(Post {
        title: "Draft: Advanced Rust Patterns".to_string(),
        content: "This is a draft post...".to_string(),
        author: Ref::new(bob),
        published: false,
        views: 0,
        created_at: 1698768000,
    })?;
    println!("   Created draft: 'Advanced Rust Patterns'\n");

    println!("4. Adding comments...");

    db.insert(Comment {
        post: Ref::new(post1),
        author: Ref::new(bob),
        content: "Great introduction! Very helpful.".to_string(),
        created_at: 1698765100,
    })?;

    db.insert(Comment {
        post: Ref::new(post1),
        author: Ref::new(carol),
        content: "Thanks for writing this!".to_string(),
        created_at: 1698765200,
    })?;

    db.insert(Comment {
        post: Ref::new(post2),
        author: Ref::new(alice),
        content: "Excellent tutorial, Bob!".to_string(),
        created_at: 1698766100,
    })?;

    println!("   Added 3 comments\n");

    println!("5. Querying published posts...");
    let published = db
        .query::<Post>()
        .filter(|p| p.published)
        .sort_by(|p| p.created_at.to_string())
        .collect();

    println!("   Found {} published posts:", published.len());
    for (id, post) in &published {
        let author = post.author.get(&db)?;
        println!("     - '{}' by @{} ({} views)",
                 post.title, author.username, post.views);
        println!("       ID: {}", id);
    }
    println!();

    println!("6. Finding popular posts (views > 1000)...");
    let popular = db
        .query::<Post>()
        .filter(|p| p.views > 1000)
        .sort_by(|p| format!("{:010}", 999999999 - p.views))  // Sort descending
        .collect();

    println!("   Popular posts:");
    for (_, post) in &popular {
        println!("     - '{}' - {} views", post.title, post.views);
    }
    println!();

    println!("7. Finding posts by author...");
    let alice_posts = db
        .query::<Post>()
        .filter(|p| p.author.id() == alice)
        .collect();

    let author_info = db.get_cloned(alice)?;
    println!("   Posts by {} (@{}):", author_info.full_name, author_info.username);
    for (_, post) in &alice_posts {
        println!("     - '{}'", post.title);
    }
    println!();

    println!("8. Finding posts with a specific tag...");
    let rust_tagged = db
        .query::<PostTag>()
        .filter(|pt| pt.tag.id() == rust_tag)
        .collect();

    println!("   Posts tagged with 'Rust':");
    for (_, post_tag) in &rust_tagged {
        let post = post_tag.post.get(&db)?;
        println!("     - '{}'", post.title);
    }
    println!();

    println!("9. Finding comments on a specific post...");
    let post1_comments = db
        .query::<Comment>()
        .filter(|c| c.post.id() == post1)
        .sort_by(|c| c.created_at.to_string())
        .collect();

    let post_info = db.get_cloned(post1)?;
    println!("   Comments on '{}':", post_info.title);
    for (_, comment) in &post1_comments {
        let author = comment.author.get(&db)?;
        println!("     - @{}: {}", author.username, comment.content);
    }
    println!();

    println!("10. Finding most commented posts...");
    let all_comments = db.query::<Comment>().collect();

    let mut comment_counts: std::collections::HashMap<Id<Post>, usize> =
        std::collections::HashMap::new();

    for (_, comment) in &all_comments {
        *comment_counts.entry(comment.post.id()).or_insert(0) += 1;
    }

    println!("   Post comment counts:");
    for (post_id, count) in &comment_counts {
        let post = db.get_cloned(*post_id)?;
        println!("     - '{}': {} comments", post.title, count);
    }
    println!();

    println!("11. Author activity summary...");
    let all_posts = db.query::<Post>().collect();
    let all_comments = db.query::<Comment>().collect();

    let mut author_stats: std::collections::HashMap<Id<Author>, (usize, usize)> =
        std::collections::HashMap::new();

    // Count posts per author
    for (_, post) in &all_posts {
        author_stats.entry(post.author.id()).or_insert((0, 0)).0 += 1;
    }

    // Count comments per author
    for (_, comment) in &all_comments {
        author_stats.entry(comment.author.id()).or_insert((0, 0)).1 += 1;
    }

    println!("   Author activity:");
    for (author_id, (posts, comments)) in &author_stats {
        let author = db.get_cloned(*author_id)?;
        println!("     - @{}: {} posts, {} comments",
                 author.username, posts, comments);
    }
    println!();

    println!("12. Finding all tags for a post...");
    let post2_tags = db
        .query::<PostTag>()
        .filter(|pt| pt.post.id() == post2)
        .collect();

    let post2_info = db.get_cloned(post2)?;
    println!("   Tags for '{}':", post2_info.title);
    for (_, post_tag) in &post2_tags {
        let tag = post_tag.tag.get(&db)?;
        println!("     - {}", tag.name);
    }
    println!();

    println!("13. Incrementing post views...");
    let before = db.get_cloned(post1)?;
    println!("   Views before: {}", before.views);

    db.update(post1, |post| {
        post.views += 1;
    })?;

    let after = db.get_cloned(post1)?;
    println!("   Views after: {}\n", after.views);

    println!("14. Publishing a draft...");
    let drafts = db
        .query::<Post>()
        .filter(|p| !p.published)
        .collect();

    println!("   Unpublished posts: {}", drafts.len());
    if let Some((draft_id, draft)) = drafts.first() {
        println!("   Publishing: '{}'", draft.title);
        db.update(*draft_id, |post| {
            post.published = true;
        })?;
        println!("   Published successfully!");
    }
    println!();

    println!("15. Complex query: Popular published Rust tutorials...");
    let rust_posts_ids: Vec<Id<Post>> = db
        .query::<PostTag>()
        .filter(|pt| pt.tag.id() == rust_tag || pt.tag.id() == tutorial_tag)
        .collect()
        .iter()
        .map(|(_, pt)| pt.post.id())
        .collect();

    let popular_rust_tutorials = db
        .query::<Post>()
        .filter(|p| p.published && p.views > 1000 && rust_posts_ids.contains(&Id::new(0)))
        .collect();

    println!("   Found {} matching posts:", popular_rust_tutorials.len());
    for (_, post) in &popular_rust_tutorials {
        println!("     - '{}' ({} views)", post.title, post.views);
    }
    println!();

    println!("16. Database statistics:");
    println!("   Authors: {}", db.query::<Author>().count());
    println!("   Posts: {}", db.query::<Post>().count());
    println!("   Published: {}", db.query::<Post>().filter(|p| p.published).count());
    println!("   Comments: {}", db.query::<Comment>().count());
    println!("   Tags: {}", db.query::<Tag>().count());
    println!("   Post-Tag associations: {}", db.query::<PostTag>().count());
    println!();

    println!("17. Finding author with most posts...");
    let all_posts = db.query::<Post>().collect();
    let mut post_counts: std::collections::HashMap<Id<Author>, usize> =
        std::collections::HashMap::new();

    for (_, post) in &all_posts {
        *post_counts.entry(post.author.id()).or_insert(0) += 1;
    }

    if let Some((top_author_id, count)) = post_counts.iter()
        .max_by_key(|(_, &count)| count)
    {
        let author = db.get_cloned(*top_author_id)?;
        println!("   Most prolific author: @{} with {} posts", author.username, count);
    }
    println!();

    println!("18. Deleting a comment...");
    let all_comments = db.query::<Comment>().collect();
    if let Some((comment_id, _)) = all_comments.first() {
        println!("   Comments before: {}", db.query::<Comment>().count());
        db.delete(*comment_id)?;
        println!("   Comments after: {}", db.query::<Comment>().count());
    }
    println!();

    println!("19. Compacting database...");
    db.compact()?;
    println!("   Database compacted successfully\n");

    println!("=== Blog example completed successfully! ===");

    Ok(())
}
