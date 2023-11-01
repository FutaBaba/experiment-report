use std::thread;
use std::sync::Arc;
fn main () {
    let mut a = String::from("Hello");
    let mut x = String::from("World");
    let mut new_a = thread::spawn(move|| {
        println!("hello from the first scoped thread");
        println!("{}", a);
        return a
    }).join().unwrap();
    let new_x = thread::spawn(move|| {
        println!("hello from the second scoped thread");
        x.push_str("xxxx");
        println!("{}", x);
        return x
    }).join().unwrap();
    println!("hello from the main thread");
    new_a.push_str("aaaa");
    println!("a = {}, x = {}", new_a, new_x);
}
