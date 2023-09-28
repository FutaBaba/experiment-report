use std::sync::Arc;
use std::thread;

fn main() {
    let s = String::from("Hello");
    let s_arc = Arc::new(s);

    thread::spawn(move || {
        let new_s = Arc::clone(&s_arc);
        new_s.len();
        println!("{}", new_s);
    });
}
