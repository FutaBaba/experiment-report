use std::thread;
use std::sync::Arc;
// use std::time::Duration;

fn main() {
    let a = Arc::new(String::from("Hello"));
    // let b = Arc::new(String::from("World"));
    // let a_clone = Arc::clone(&a);

    thread::scope(|s| {
        let b = a;
        let t1 = s.spawn(|| {
            println!("{} {}", a);
        });

        t1.join();
        let t2 = s.spawn(|| {
            a.push_str("bbbb");
            println!("{}", a);
        });

    });
}