use std::thread;
use std::sync::mpsc::channel;

fn main() {
    // Create a simple streaming channel
    let (tx, rx) = channel();
    thread::spawn(move|| {
        tx.send(10).unwrap();
    });
    assert_eq!(rx.recv().unwrap(), 10);
}