use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::collections::VecDeque;

fn main() {
    let lock = Arc::new(Mutex::new(true));
    let lock_clone = Arc::clone(&lock);
    let queue = Arc::new(Mutex::new(VecDeque::with_capacity(1)));
    let queue_clone = Arc::clone(&queue);

    let producer = thread::spawn(move || {
        loop {
            let mut lock_producer = lock_clone.lock().unwrap();
            let mut queue_producer = queue_clone.lock().unwrap();
            if *lock_producer == true {
                queue_producer.push_front(30);
                println!("produce data");
                thread::sleep(Duration::from_secs(1));
                *lock_producer = false;
            }
        }
    });

    let consumer = thread::spawn(move || {
        loop {
            let mut lock_consumer = lock.lock().unwrap();
            let mut queue_consumer = queue.lock().unwrap();
            if *lock_consumer == false {
                let data = queue_consumer.pop_back().unwrap();
                println!("consume {}", data);
                thread::sleep(Duration::from_secs(1));
                *lock_consumer = true;
            }
        }
    });

    let _ = producer.join();
    let _ = consumer.join();
}