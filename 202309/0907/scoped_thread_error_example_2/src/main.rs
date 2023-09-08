use std::thread;
// use std::sync::Arc;
// use std::time::Duration;

fn main() {
    let mut a = String::from("Hello");
    // let b = &mut a;

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            println!("{}", a);
        });

        t1.join();
        let t2 = s.spawn(|| {
            a.push_str("bbbb");
            println!("{}", a);
        });

    });
}