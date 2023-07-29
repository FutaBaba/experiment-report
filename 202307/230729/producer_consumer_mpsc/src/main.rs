use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::collections::VecDeque;

fn main() {
    let lock = Arc::new(Mutex::new(true));
    let lock_clone_1 = Arc::clone(&lock);
    let lock_clone_2 = Arc::clone(&lock);
    let queue = Arc::new(Mutex::new(VecDeque::with_capacity(2)));
    let queue_clone_1 = Arc::clone(&queue);
    let queue_clone_2 = Arc::clone(&queue);

    let producer_1 = thread::spawn(move || {
        loop {
            let mut lock_producer = lock_clone_1.lock().unwrap();
            let mut queue_producer = queue_clone_1.lock().unwrap();
            if *lock_producer == true {
                queue_producer.push_front(30);
                println!("produce 30");
                thread::sleep(Duration::from_secs(1));
                *lock_producer = false;
            }
        }
    });

    let producer_2 = thread::spawn(move || {
        loop {
            let mut lock_producer = lock_clone_2.lock().unwrap();
            let mut queue_producer = queue_clone_2.lock().unwrap();
            if *lock_producer == true {
                queue_producer.push_front(40);
                println!("produce 40");
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

    let _ = producer_1.join();
    let _ = producer_2.join();
    let _ = consumer.join();
}