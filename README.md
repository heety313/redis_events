# Redis Events

Redis Events is a Rust library that provides a simple and efficient way to watch for changes in Redis hash fields. It allows you to register callbacks for specific hash:field combinations and automatically triggers these callbacks when the values change. Uses polling under the hood allowing you to set polling rate based on your needs.

## Features

- No async needed, just regular functions
- Support for multiple hash:field combinations
- Thread-safe

## Prerequisites

Before using this library, make sure you have:

1. Rust installed on your system
2. Redis server running (default: `redis://127.0.0.1/`)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
redis_events = "0.1.0"
```

## Usage

Run `cargo run --example example` to see an example of the library in action. Make sure you have a Redis server running on `redis://127.0.0.1/` or set the `REDIS_URL` environment variable to your Redis server URL. Also add the hash and key you want to watch to the redis server before running the example.

```rust
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
}
```