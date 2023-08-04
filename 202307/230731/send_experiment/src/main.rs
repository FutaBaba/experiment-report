use std::sync::mpsc::channel;
use std::thread;

fn main(){
    let (tx, rx) = channel();
    let s = String::from("Hello");

    thread::spawn(move || { 
        tx.send(s).unwrap();
        println!("{}", s);
    });
    // This send will fail because the receiver is gone
    rx.recv().unwrap();
}