#![feature(thread_spawn_unchecked)]
use std::thread;
// use std::sync::Arc;
use std::time::Duration;
use std::thread::JoinHandle;
use std::thread::ScopedJoinHandle;

fn f<T> (t: JoinHandle<T>) {
    t.join();
}

fn g (a: &String) -> Result<JoinHandle<()>, std::io::Error>{
    let builder = thread::Builder::new();
    unsafe{
        let t1 = builder.spawn_unchecked(|| {
            thread::sleep(Duration::from_secs(1));
            println!("{}", &a);
        });
        
       return t1;
    }
}

fn main() {

    let builder2 = thread::Builder::new();

    // let mut a = String::from("Hello");

    unsafe {
        let mut a = String::from("Hello");
        // let t1 = builder.spawn_unchecked(|| {
        //     println!("{}", &a);
        // });
        // f(t1.unwrap());
        g(&a);//.unwrap().join();
    
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