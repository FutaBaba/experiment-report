use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::Builder;

fn my_spawn (a: &Arc<String>) -> thread::JoinHandle<()> {
    let t = thread::spawn(|| {
        println!("{}", a);
    });       
   return t
}

fn main() {
    let s = Arc::new(String::from("Hello"));
    let s1 = s.clone(); // +1
    let s2 = s.clone(); // +1
    
    let t1 = my_spawn(&s1); // -1
    let t2 = my_spawn(&s2); // -1

    t1.join();
    t2.join();
    let mut s4 = Arc::into_inner(s).unwrap();
    s4.push_str("World");
    println!("{}", s4);
} 