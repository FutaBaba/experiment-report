use std::thread;
use std::sync::Arc;
// use std::time::Duration;

fn main() {
    let builder = thread::Builder::new();

    let a = Arc::new(String::from("Hello"));
    let b = Arc::new(String::from("World"));
    let a_clone = Arc::clone(&a);

    #![feature(thread_spawn_unchecked)]
    unsafe{
        builder.spawn_unchecked(move || {
            println!("{}, {}", a, b);
        }).unwrap();
    }
}