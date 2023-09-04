use std::sync::Arc;
use std::thread;

fn main() {
    let mut s = Arc::new(String::from("Hello"));

    for _ in 0..10 {
        let s_clone = Arc::clone(&s);

        thread::spawn(move || {
            println!("{}", s_clone.is_empty());
        });
    }
    // s.push_str("d");
}