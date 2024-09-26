use redis::{Commands, RedisResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct RedisEvents {
    registered: Arc<Mutex<HashMap<(String, String), Box<dyn Fn(&str, &str, String) + Send + Sync>>>>,
    redis_url: String,
    polling_rate: u64,
    previous_values: Arc<Mutex<HashMap<(String, String), String>>>,
}

impl RedisEvents {
    pub fn new(polling_rate: Option<u64>) -> Arc<Self> {
        Arc::new(Self {
            registered: Arc::new(Mutex::new(HashMap::new())),
            redis_url: "redis://127.0.0.1/".to_string(),
            polling_rate: polling_rate.unwrap_or(1000),
            previous_values: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn new_with_url(redis_url: &str, polling_rate: Option<u64>) -> Arc<Self> {
        Arc::new(Self {
            registered: Arc::new(Mutex::new(HashMap::new())),
            redis_url: redis_url.to_string(),
            polling_rate: polling_rate.unwrap_or(1000),
            previous_values: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn register<F>(&self, hash: &str, field: &str, callback: F)
    where
        F: Fn(&str, &str, String) + 'static + Send + Sync,
    {
        self.registered.lock().unwrap().insert(
            (hash.to_string(), field.to_string()),
            Box::new(callback),
        );
    }
    
    pub fn get_value(&self, hash: &str, field: &str) -> Option<String> {
        self.previous_values
            .lock()
            .unwrap()
            .get(&(hash.to_string(), field.to_string()))
            .cloned()
    }

    pub fn start(self: &Arc<Self>) {
        let events = Arc::clone(self);
        
        thread::spawn(move || {
            let client = redis::Client::open(events.redis_url.as_str()).unwrap();
            let mut con = client.get_connection().unwrap();

            loop {
                for ((hash, field), callback) in events.registered.lock().unwrap().iter() {
                    let result: RedisResult<String> = con.hget(hash, field);
                    match result {
                        Ok(value) => {
                            let key = (hash.clone(), field.clone());
                            let mut previous_values = events.previous_values.lock().unwrap();
                            let prev_value = previous_values.get(&key);

                            if prev_value.map_or(true, |v| v != &value) {
                                callback(hash, field, value.clone());
                                previous_values.insert(key, value);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error fetching {} {}: {}", hash, field, e);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(events.polling_rate));
            }
        });
    }
}
