#![feature(thread_spawn_unchecked)]
use std::thread;
use std::sync::Arc;
use std::time::Duration;
use std::thread::JoinHandle;
use std::thread::ScopedJoinHandle;

fn g (a: Arc<String>) -> Result<JoinHandle<()>, std::io::Error>{
    let builder = thread::Builder::new();
    unsafe{
        let t1 = thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            println!("{}", a);
        });
        
       return Ok(t1);
    }
}

fn main() {

    // let builder2 = thread::Builder::new();

    let mut a = String::from("Hello");

    unsafe {
        let mut new_a = Arc::new(a);
        let mut new_a_2 = Arc::clone(&new_a);
        g(new_a);
    
        let t2 = thread::spawn(move || {
            // a.push_str("bbbb");
            println!("{}", new_a_2);
        });
    }

    thread::sleep(Duration::from_secs(2));
}