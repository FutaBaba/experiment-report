use std::sync::Arc;
use std::thread;

fn main() {
    let x: Arc<String> = Arc::new(String::from("Hello1"));
    let x_clone = Arc::clone(&x);

    let thread_1: thread::JoinHandle<()> = thread::spawn(move || {
        println!("{}", x);
    });

    let thread_2: thread::JoinHandle<()> = thread::spawn(move || {
        println!("{}", x_clone);
    });

    let _ = thread_1.join();
    let _ = thread_2.join();
}
