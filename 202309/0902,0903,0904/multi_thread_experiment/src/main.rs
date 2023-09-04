use std::thread;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let a = Arc::new(String::from("Hello"));
    let b = Arc::new(String::from("World"));
    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);

    let t1 = thread::spawn(move || {
        println!("{} {}", a, b);
    });

    t1.join().unwrap();
    
    let t2 = thread::spawn(move || {
        thread::sleep(Duration::from_secs(10));
        println!("{}", a);
    });

    t2.join().unwrap();
}
