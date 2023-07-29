use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let lock = Arc::new(Mutex::new(true));
    let lock_clone = Arc::clone(&lock);

    let producer = thread::spawn(move || {
        loop {
            let mut lock_producer = lock_clone.lock().unwrap();
            if *lock_producer == true {
                println!("produce data");
                thread::sleep(Duration::from_secs(1));
                *lock_producer = false;
            }
        }
    });

    let consumer = thread::spawn(move || {
        loop {
            let mut lock_consumer = lock.lock().unwrap();
            if *lock_consumer == false {
                println!("consume data");
                thread::sleep(Duration::from_secs(1));
                *lock_consumer = true;
            }
        }
    });

    let _ = producer.join();
    let _ = consumer.join();
}