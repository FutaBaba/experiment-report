use std::thread;
// use std::sync::Arc;
// use std::time::Duration;
// use std::thread::JoinHandle;
use std::thread::ScopedJoinHandle;

fn f<T> (t: ScopedJoinHandle<T>) {
    t.join();
}

fn main() {
    let mut a = String::from("Hello");
    let b = &mut a;

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            println!("{}", b);
        });
        f(t1);
    
        let t2 = s.spawn(|| {
            a.push_str("bbbb");
            println!("{}", a);
        });

    });

    // a.push_str("xxx");
    // println!("{}", a);
}