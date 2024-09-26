use redis_events::RedisEvents;

fn sample_callback(hash: &str, field: &str, value: String) {
    println!("Got a new value for {}:{} = {}", hash, field, value);
}

fn main() {
    let events = RedisEvents::new(None);
    
    events.register("user", "name", |hash, field, value| {
        sample_callback(hash, field, value);
    });
    
    events.start();
    
    std::thread::sleep(std::time::Duration::from_secs(1));

    if let Some(value) = events.get_value("user", "name") {
        println!("Current value: {}", value);
    } else {
        println!("Value not available yet");
    }
    
    // Keep the main thread running
    std::thread::park();
}
