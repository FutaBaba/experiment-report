use std::thread;
// use std::sync::Arc;
// use std::time::Duration;

fn main() {
    let mut a = String::from("Hello");
    let b = &a;

    let s = thread::scope(|s| {
        return s;
    });
    thread::scope(|s| {
        let t1 = s.spawn(|| {
            println!("{}", b);
        });
    });

    println!("{}", b);
    

    thread::scope(|s1| {
        let t2 = s1.spawn(|| {
            a.push_str("bbbb");
            println!("{}", a);
        });
    });

    println!("{}", a);

}