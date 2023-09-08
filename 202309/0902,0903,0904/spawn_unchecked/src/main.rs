// #![feature(thread_spawn_unchecked)]
// use std::thread;

// fn main() {
//     let builder = thread::Builder::new();

//     let x = 1;
//     let thread_x = &x;

//     let handler = unsafe {
//         builder.spawn_unchecked(move || {
//             println!("x = {}", *thread_x);
//         }).unwrap()
//     };

//     // caller has to ensure `join()` is called, otherwise
//     // it is possible to access freed memory if `x` gets
//     // dropped before the thread closure is executed!
//     handler.join().unwrap();
// }

#![feature(thread_spawn_unchecked)]
use std::thread;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let builder = thread::Builder::new();
    let builder2 = thread::Builder::new();

    let mut a = String::from("Hello");
    let b = Arc::new(String::from("World"));
    // let a_clone = Arc::clone(&a);

    unsafe{
        let mut a2 = &mut a;
        builder.spawn_unchecked(move || {
            // thread::sleep(Duration::from_secs(1));
            a2.push_str("aaaa");
            println!("{}", a2);
        }).unwrap().join();

        builder2.spawn_unchecked(move || {
            thread::sleep(Duration::from_secs(1));
            a.push_str("bbbb");
            println!("{}", a);
        }).unwrap().join();
    }
}