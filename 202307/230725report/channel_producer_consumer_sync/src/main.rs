use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::sync_channel(0);

    let producer = thread::spawn(move || {
        for _ in 0..5{
            let data = 32;
            println!("produce {}", data);
            println!("send {}", data);
            tx.send(data).unwrap();
            // thread::sleep(Duration::from_secs(1));
        }
    });

    let consumer = thread::spawn(move || {
        for _ in 0..5{
            println!("receive {}", rx.recv().unwrap());
        }
    });

    let _ = producer.join();
    let _ = consumer.join();
}