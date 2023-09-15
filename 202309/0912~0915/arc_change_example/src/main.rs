#![feature(thread_spawn_unchecked)]
use std::thread;
// use std::sync::Arc;
use std::time::Duration;
use std::thread::JoinHandle;
use std::thread::ScopedJoinHandle;
use std::sync::Arc;

fn f<T> (t: JoinHandle<T>) {
    t.join();
}

fn g (a: &Arc<&String>) -> Result<JoinHandle<()>, std::io::Error>{
    let builder = thread::Builder::new();
    let new_a = a.clone();
    unsafe{
        let t1 = builder.spawn_unchecked(|| {
            thread::sleep(Duration::from_secs(1));
            println!("{}", &new_a);
        });
        
       return t1;
    }
}

fn main() {

    let builder2 = thread::Builder::new();

    let mut a = String::from("Hello");

    unsafe {
        let arc_a = Arc::new(&a);
        // let t1 = builder.spawn_unchecked(|| {
        //     println!("{}", &a);
        // });
        // f(t1.unwrap());
        g(&arc_a);//.unwrap().join();
    
        let t2 = builder2.spawn_unchecked(|| {
            a.push_str("bbbb");
            println!("{}", a);
        }).unwrap().join();

        drop(a);
    }

    // a.push_str("xxx");
    // println!("{}", a);
    thread::sleep(Duration::from_secs(2));
}