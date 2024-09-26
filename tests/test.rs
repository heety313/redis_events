use redis::{Client, Commands};
use redis_events::RedisEvents;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn setup_redis() -> Client {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    Client::open(redis_url).expect("Failed to connect to Redis")
}

#[test]
fn test_single_field_change() {
    let client = setup_redis();
    let mut con = client.get_connection().unwrap();

    let events = RedisEvents::new(Some(100)); // 100ms polling rate
    let value_received = Arc::new(Mutex::new(String::new()));
    let value_clone = Arc::clone(&value_received);

    events.register("test_single_field_change", "field1", move |_, _, value| {
        let mut received = value_clone.lock().unwrap();
        *received = value;
    });

    events.start();

    // Set initial value
    let _: () = con.hset("test_single_field_change", "field1", "initial").unwrap();
    thread::sleep(Duration::from_millis(200));

    // Change value
    let _: () = con.hset("test_single_field_change", "field1", "changed").unwrap();
    thread::sleep(Duration::from_millis(200));

    assert_eq!(*value_received.lock().unwrap(), "changed");

    // Clean up
    let _: () = con.del("test_single_field_change").unwrap();
}

#[test]
fn test_multiple_fields() {
    let client = setup_redis();
    let mut con = client.get_connection().unwrap();

    let events = RedisEvents::new(Some(100));
    let values_received = Arc::new(Mutex::new(vec![String::new(), String::new()]));

    let values_clone1 = Arc::clone(&values_received);
    events.register("test_multiple_fields", "field1", move |_, _, value| {
        let mut received = values_clone1.lock().unwrap();
        received[0] = value;
    });

    let values_clone2 = Arc::clone(&values_received);
    events.register("test_multiple_fields", "field2", move |_, _, value| {
        let mut received = values_clone2.lock().unwrap();
        received[1] = value;
    });

    events.start();

    // Set initial values
    let _: () = con.hset("test_multiple_fields", "field1", "value1").unwrap();
    let _: () = con.hset("test_multiple_fields", "field2", "value2").unwrap();
    thread::sleep(Duration::from_millis(200));

    // Change values
    let _: () = con.hset("test_multiple_fields", "field1", "new1").unwrap();
    let _: () = con.hset("test_multiple_fields", "field2", "new2").unwrap();
    thread::sleep(Duration::from_millis(200));

    let received = values_received.lock().unwrap();
    assert_eq!(received[0], "new1");
    assert_eq!(received[1], "new2");

    // Clean up
    let _: () = con.del("test_multiple_fields").unwrap();
}

#[test]
fn test_get_value() {
    let client = setup_redis();
    let mut con = client.get_connection().unwrap();

    let events = RedisEvents::new(Some(10));
    events.register("test_get_value", "field", |_, _, _| {});
    events.start();

    // Set value
    let _: () = con.hset("test_get_value", "field", "test_value").unwrap();
    thread::sleep(Duration::from_millis(400));

    assert_eq!(events.get_value("test_get_value", "field"), Some("test_value".to_string()));

    // Clean up
    let _: () = con.del("test_get_value").unwrap();
}

#[test]
fn test_non_existent_field() {
    let events = RedisEvents::new(Some(100));
    events.register("test_non_existent_field", "non_existent", |_, _, _| {});
    events.start();

    thread::sleep(Duration::from_millis(200));

    assert_eq!(events.get_value("test_non_existent_field", "non_existent"), None);
}

#[test]
fn test_multiple_changes() {
    let client = setup_redis();
    let mut con = client.get_connection().unwrap();

    let events = RedisEvents::new(Some(100));
    let changes = Arc::new(Mutex::new(Vec::new()));
    let changes_clone = Arc::clone(&changes);

    events.register("test_multiple_changes", "field", move |_, _, value| {
        let mut changes = changes_clone.lock().unwrap();
        changes.push(value);
    });

    events.start();

    // Set initial value
    let _: () = con.hset("test_multiple_changes", "field", "initial").unwrap();
    thread::sleep(Duration::from_millis(150));

    // Change value multiple times
    for i in 1..=5 {
        let _: () = con.hset("test_multiple_changes", "field", format!("value{}", i)).unwrap();
        thread::sleep(Duration::from_millis(150));
    }

    let changes = changes.lock().unwrap();
    assert_eq!(changes.len(), 6); // initial + 5 changes
    assert_eq!(changes[0], "initial");
    assert_eq!(changes[5], "value5");

    // Clean up
    let _: () = con.del("test_multiple_changes").unwrap();
}

#[test]
fn test_custom_redis_url() {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let events = RedisEvents::new_with_url(&redis_url, Some(100));
    let value_received = Arc::new(Mutex::new(String::new()));
    let value_clone = Arc::clone(&value_received);

    events.register("test_custom_redis_url", "custom_url", move |_, _, value| {
        let mut received = value_clone.lock().unwrap();
        *received = value;
    });

    events.start();

    let client = Client::open(redis_url).unwrap();
    let mut con = client.get_connection().unwrap();

    let _: () = con.hset("test_custom_redis_url", "custom_url", "custom_value").unwrap();
    thread::sleep(Duration::from_millis(200));

    assert_eq!(*value_received.lock().unwrap(), "custom_value");

    // Clean up
    let _: () = con.del("test_custom_redis_url").unwrap();
}
