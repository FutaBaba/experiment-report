use std::thread;
// use std::sync::Arc;
fn main () {
    let mut a = String::from("Hello");
    let b = String::from("bbbbbb");
    let mut x = String::from("World");
    let mut a2 = thread::spawn(move|| {
        println!("hello from the first scoped thread");
        println!("{}", a);
        return a
    }).join().unwrap();
    let x2 = thread::spawn(move|| {
        println!("hello from the second scoped thread");
        x.push_str("xxxx");
        println!("{}", x);
        return (x, b)
    }).join().unwrap();
    println!("hello from the main thread");
    a2.push_str("aaaa");
    let (x3, b2) = x2;
    println!("a = {}, x = {}, b = {}", a2, x3, b2);
}
