use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let lock = Arc::new(Mutex::new(true));
    let lock_clone = Arc::clone(&lock);

    let producer = thread::spawn(move || {
        let mut lock_producer = lock_clone.lock().unwrap();
        println!("lock_producer is {}", *lock_producer);
        if *lock_producer == true {
            println!("produce data");
            *lock_producer = false;
            println!("lock_producer is {}", *lock_producer);
        }
    });

    let consumer = thread::spawn(move || {
        let mut lock_consumer = lock.lock().unwrap();
        println!("lock_consumer is {}", *lock_consumer);
        if *lock_consumer == false {
            println!("consume data");
            *lock_consumer = true;
            println!("lock_consumer is {}", *lock_consumer);
        }
    });

    producer.join();
    consumer.join();
}