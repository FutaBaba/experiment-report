use std::thread;
use std::sync::mpsc::sync_channel;

fn main() {
    let (tx, rx) = sync_channel::<i32>(0);
    thread::spawn(move|| {
        // This will wait for the parent thread to start receiving
        tx.send(53).unwrap();
    });
    println!("{}",rx.recv().unwrap());
}
