use std::thread;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let a = Arc::new(String::from("Hello"));
    let b = Arc::new(String::from("World"));
    let a_clone = Arc::clone(&a);

    thread::scope(|s| {
        s.spawn(|| {
            println!("{} {}", a, b);
        });

        println!("{}",b);

        s.spawn(|| {
            thread::sleep(Duration::from_secs(5));
            println!("{}", a);
        });

        println!("{}",b);
    });
}