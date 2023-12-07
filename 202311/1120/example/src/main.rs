#![feature(thread_spawn_unchecked)]
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::Builder;

fn my_spawn (a: Arc<Mutex<String>>) -> thread::JoinHandle<()> {
    let t1 = thread::spawn(move || {
        println!("{}", a.lock().unwrap());
    });       
   return t1
}

// fn main() {
//     let s = Arc::new(String::from("Hello"));
//     let s1 = s.clone();
//     let s2 = s.clone();
    
//     let t1 = my_spawn(&s1);//2のやつを渡す
//     let t2 = my_spawn(&s2);

//     let _ = t1.unwrap().join();
//     let _ = t2.unwrap().join();
//     drop(s1);
//     drop(s2);
//     let mut s3 = Arc::into_inner(s).unwrap();
//     s3.push_str("World");
//     println!("{}", s3);
// }

fn main() {
    let mut s = Arc::new(Mutex::new(String::from("Hello")));
    let s1 = s.clone();
    let s2 = s.clone();
    let t1 = my_spawn(s1);
    let t2 = my_spawn(s2);
    t1.join().unwrap();
    t2.join().unwrap();
    s.lock().unwrap().push_str("World");
    println!("{}", s.lock().unwrap());
}
    