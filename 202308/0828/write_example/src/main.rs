use std::sync::Arc;
use std::thread;
use std::sync::Mutex;

fn main() {
    let s = Arc::new(Mutex::new(String::from("Hello")));

    for _ in 0..10 {
        let s_clone = Arc::clone(&s);

        let thread = thread::spawn(move || {
            let mut s_unlock = s_clone.lock().unwrap();
            println!("{}", s_unlock.is_empty());
        });
        thread.join().unwrap();
    }
    let s_clone = Arc::clone(&s);
    let mut s_clone = s_clone.lock().unwrap();
    s_clone.push_str("d");
    println!("{}", s_clone);
}