use serde::{Deserialize, Serialize};
use struct_db::{Database, Id, Ref, Table};

/// Customer in the e-commerce system
#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct Customer {
    name: String,
    email: String,
    address: String,
}

/// Product available for purchase
#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct Product {
    name: String,
    description: String,
    price: f64,
    stock: u32,
}

/// Order placed by a customer
#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct Order {
    customer: Ref<Customer>,
    status: String,  // "pending", "shipped", "delivered"
    total: f64,
}

/// Individual item in an order
#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct OrderItem {
    order: Ref<Order>,
    product: Ref<Product>,
    quantity: u32,
    price: f64,  // Price at time of order
}

fn main() -> anyhow::Result<()> {
    println!("=== E-Commerce System Example ===\n");

    // Setup database
    let db = Database::open("./data/ecommerce")?

        .register::<Customer>()

        .register::<Product>()

        .register::<Order>()

        .register::<OrderItem>()

        .build()?;println!("1. Adding customers...");
    let customer1 = db.insert(Customer {
        name: "Sarah Connor".to_string(),
        email: "sarah@example.com".to_string(),
        address: "123 Tech Street, San Francisco, CA".to_string(),
    })?;
    println!("   Created customer: {} (ID: {})", "Sarah Connor", customer1);

    let customer2 = db.insert(Customer {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        address: "456 Main Ave, New York, NY".to_string(),
    })?;
    println!("   Created customer: {} (ID: {})\n", "John Doe", customer2);

    println!("2. Adding products...");
    let laptop = db.insert(Product {
        name: "Laptop Pro 15".to_string(),
        description: "High-performance laptop".to_string(),
        price: 1299.99,
        stock: 50,
    })?;
    println!("   Added: Laptop Pro 15 - ${}", 1299.99);

    let mouse = db.insert(Product {
        name: "Wireless Mouse".to_string(),
        description: "Ergonomic wireless mouse".to_string(),
        price: 29.99,
        stock: 200,
    })?;
    println!("   Added: Wireless Mouse - ${}", 29.99);

    let keyboard = db.insert(Product {
        name: "Mechanical Keyboard".to_string(),
        description: "RGB mechanical keyboard".to_string(),
        price: 149.99,
        stock: 100,
    })?;
    println!("   Added: Mechanical Keyboard - ${}\n", 149.99);

    println!("3. Creating orders...");

    // Order 1: Sarah buys laptop and mouse
    let order1 = db.insert(Order {
        customer: Ref::new(customer1),
        status: "pending".to_string(),
        total: 1329.98,
    })?;
    println!("   Created order: {} for Sarah Connor", order1);

    db.insert(OrderItem {
        order: Ref::new(order1),
        product: Ref::new(laptop),
        quantity: 1,
        price: 1299.99,
    })?;

    db.insert(OrderItem {
        order: Ref::new(order1),
        product: Ref::new(mouse),
        quantity: 1,
        price: 29.99,
    })?;

    // Order 2: John buys keyboard and mouse
    let order2 = db.insert(Order {
        customer: Ref::new(customer2),
        status: "shipped".to_string(),
        total: 179.98,
    })?;
    println!("   Created order: {} for John Doe\n", order2);

    db.insert(OrderItem {
        order: Ref::new(order2),
        product: Ref::new(keyboard),
        quantity: 1,
        price: 149.99,
    })?;

    db.insert(OrderItem {
        order: Ref::new(order2),
        product: Ref::new(mouse),
        quantity: 1,
        price: 29.99,
    })?;

    println!("4. Querying orders by status...");
    let pending_orders = db
        .query::<Order>()?
        .filter(|o| o.status == "pending")
        .collect();

    println!("   Pending orders: {}", pending_orders.len());
    for (id, order) in &pending_orders {
        let customer = order.customer.get(&db)?;
        println!("     - Order {}: {} (total: ${:.2})",
                 id, customer.name, order.total);
    }
    println!();

    println!("5. Finding all items in an order...");
    let order1_items = db
        .query::<OrderItem>()?
        .filter(|item| item.order.id() == order1)
        .collect();

    println!("   Order {} contains {} items:", order1, order1_items.len());
    for (_, item) in &order1_items {
        let product = item.product.get(&db)?;
        println!("     - {} x{} @ ${:.2}",
                 product.name, item.quantity, item.price);
    }
    println!();

    println!("6. Finding all orders for a customer...");
    let sarah_orders = db
        .query::<Order>()?
        .filter(|order| order.customer.id() == customer1)
        .collect();

    println!("   Sarah Connor has {} orders:", sarah_orders.len());
    for (id, order) in &sarah_orders {
        println!("     - Order {}: {} (${:.2})",
                 id, order.status, order.total);
    }
    println!();

    println!("7. Finding popular products (by order count)...");
    let all_items = db.query::<OrderItem>()?.collect();

    // Count orders per product
    let mut product_counts: std::collections::HashMap<Id<Product>, u32> =
        std::collections::HashMap::new();

    for (_, item) in &all_items {
        *product_counts.entry(item.product.id()).or_insert(0) += 1;
    }

    println!("   Product popularity:");
    for (product_id, count) in &product_counts {
        let product = db.get(*product_id)?;
        println!("     - {}: {} orders", product.name, count);
    }
    println!();

    println!("8. Updating order status...");
    let order_before = db.get(order1)?;
    println!("   Order {} status before: {}", order1, order_before.status);

    db.update(order1, |order| {
        order.status = "shipped".to_string();
    })?;

    let order_after = db.get(order1)?;
    println!("   Order {} status after: {}\n", order1, order_after.status);

    println!("9. Updating product stock after purchase...");
    let laptop_before = db.get(laptop)?;
    println!("   Laptop stock before: {}", laptop_before.stock);

    db.update(laptop, |product| {
        product.stock -= 1;
    })?;

    let laptop_after = db.get(laptop)?;
    println!("   Laptop stock after: {}\n", laptop_after.stock);

    println!("10. Finding low-stock products...");
    let low_stock = db
        .query::<Product>()?
        .filter(|p| p.stock < 60)
        .sort_by_key(|p| &p.stock)
        .collect();

    println!("   Products with stock < 60:");
    for (id, product) in &low_stock {
        println!("     - {}: {} units (ID: {})",
                 product.name, product.stock, id);
    }
    println!();

    println!("11. Calculating total revenue...");
    let all_orders = db.query::<Order>()?.collect();
    let total_revenue: f64 = all_orders.iter()
        .map(|(_, order)| order.total)
        .sum();
    println!("   Total revenue: ${:.2}\n", total_revenue);

    println!("12. Finding high-value orders...");
    let high_value = db
        .query::<Order>()?
        .filter(|o| o.total > 500.0)
        .sort_by(|a, b| b.total.partial_cmp(&a.total).unwrap())  // Sort descending
        .collect();

    println!("   Orders over $500:");
    for (id, order) in &high_value {
        let customer = order.customer.get(&db)?;
        println!("     - Order {}: {} - ${:.2}",
                 id, customer.name, order.total);
    }
    println!();

    println!("13. Customer analytics...");

    // Calculate total spent per customer
    let all_orders = db.query::<Order>()?.collect();
    let mut customer_spending: std::collections::HashMap<Id<Customer>, f64> =
        std::collections::HashMap::new();

    for (_, order) in &all_orders {
        *customer_spending.entry(order.customer.id()).or_insert(0.0) += order.total;
    }

    println!("   Customer spending:");
    for (customer_id, total) in &customer_spending {
        let customer = db.get(*customer_id)?;
        println!("     - {}: ${:.2}", customer.name, total);
    }
    println!();

    println!("14. Database statistics:");
    println!("   Customers: {}", db.query::<Customer>()?.count());
    println!("   Products: {}", db.query::<Product>()?.count());
    println!("   Orders: {}", db.query::<Order>()?.count());
    println!("   Order Items: {}", db.query::<OrderItem>()?.count());
    println!();

    println!("15. Compacting database...");
    db.compact()?;
    println!("   Database compacted successfully\n");

    println!("=== E-Commerce example completed successfully! ===");

    Ok(())
}
