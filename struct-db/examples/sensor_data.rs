use serde::{Deserialize, Serialize};
use struct_db::{Database, Id, Ref, Table};

// Define our domain models

#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct Sensor {
    location: String,
    sensor_type: String,
}

#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct Reading {
    timestamp: u64,
    value: f64,
    sensor: Ref<Sensor>,
}

#[derive(Table, Serialize, Deserialize, Clone, Debug)]
struct Alert {
    message: String,
    reading: Ref<Reading>,
    severity: String,
}

fn main() -> anyhow::Result<()> {
    println!("=== Struct-DB Sensor Data Example ===\n");

    // Open or create database
    let db = Database::open("./data/sensor_db")?;

    // Register types for WAL replay
    db.register:::<Sensor>();
    db.register:::<Reading>();
    db.register:::<Alert>();

    println!("1. Inserting sensors...");

    // Insert sensors
    let sensor1_id = db.insert(Sensor {
        location: "Building A - Room 101".to_string(),
        sensor_type: "Temperature".to_string(),
    })?;
    println!("   Created sensor: {}", sensor1_id);

    let sensor2_id = db.insert(Sensor {
        location: "Building A - Room 102".to_string(),
        sensor_type: "Humidity".to_string(),
    })?;
    println!("   Created sensor: {}", sensor2_id);

    println!("\n2. Recording readings...");

    // Insert readings
    let reading1_id = db.insert(Reading {
        timestamp: 1698765432,
        value: 22.5,
        sensor: Ref::new(sensor1_id),
    })?;
    println!("   Recorded reading: {} (value: 22.5°C)", reading1_id);

    let reading2_id = db.insert(Reading {
        timestamp: 1698765492,
        value: 28.3,
        sensor: Ref::new(sensor1_id),
    })?;
    println!("   Recorded reading: {} (value: 28.3°C)", reading2_id);

    let reading3_id = db.insert(Reading {
        timestamp: 1698765552,
        value: 65.2,
        sensor: Ref::new(sensor2_id),
    })?;
    println!("   Recorded reading: {} (value: 65.2%)", reading3_id);

    println!("\n3. Creating alerts...");

    // Create an alert for high temperature
    let alert_id = db.insert(Alert {
        message: "Temperature exceeds threshold!".to_string(),
        reading: Ref::new(reading2_id),
        severity: "High".to_string(),
    })?;
    println!("   Created alert: {}", alert_id);

    println!("\n4. Querying data...");

    // Query all sensors
    println!("   All sensors:");
    let sensors = db.query::<Sensor>().collect();
    for (id, sensor) in &sensors {
        println!("     - {}: {} ({})", id, sensor.location, sensor.sensor_type);
    }

    // Query readings with filters
    println!("\n   Readings with value > 25:");
    let high_readings = db
        .query::<Reading>()
        .filter(|r| r.value > 25.0)
        .collect();

    for (id, reading) in &high_readings {
        // Access the referenced sensor
        let sensor = reading.sensor.get(&db)?;
        println!(
            "     - {}: {} at {} (sensor: {})",
            id, reading.value, reading.timestamp, sensor.location
        );
    }

    // Query with sorting
    println!("\n   All readings sorted by timestamp:");
    let sorted_readings = db
        .query::<Reading>()
        .sort_by(|r| r.timestamp.to_string())
        .collect();

    for (id, reading) in &sorted_readings {
        println!("     - {}: {} at {}", id, reading.value, reading.timestamp);
    }

    // Query with limit
    println!("\n   First 2 readings:");
    let limited_readings = db.query::<Reading>().limit(2).collect();

    for (id, reading) in &limited_readings {
        println!("     - {}: {}", id, reading.value);
    }

    println!("\n5. Updating data...");

    // Update a sensor
    db.update(sensor1_id, |sensor| {
        sensor.location = "Building B - Room 201".to_string();
    })?;
    println!("   Updated sensor location");

    let updated_sensor = db.get_cloned(sensor1_id)?;
    println!("   New location: {}", updated_sensor.location);

    println!("\n6. Deleting data...");

    // Delete an alert
    db.delete(alert_id)?;
    println!("   Deleted alert: {}", alert_id);

    let remaining_alerts = db.query::<Alert>().count();
    println!("   Remaining alerts: {}", remaining_alerts);

    println!("\n7. Compacting WAL...");

    // Compact the WAL
    db.compact()?;
    println!("   WAL compacted successfully");

    println!("\n8. Database statistics:");
    println!("   Sensors: {}", db.query::<Sensor>().count());
    println!("   Readings: {}", db.query::<Reading>().count());
    println!("   Alerts: {}", db.query::<Alert>().count());

    println!("\n=== Example completed successfully! ===");

    Ok(())
}
