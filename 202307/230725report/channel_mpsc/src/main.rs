use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::sync_channel(0);

    let tx_1 = tx.clone();
    let producer_1 = thread::spawn(move || {
        for _ in 0..5{
            let data = 32;
            println!("produced by 1 {}", data);
            tx_1.send(data).unwrap();
            // thread::sleep(Duration::from_secs(1));
        }
    });

    let producer_2 = thread::spawn(move || {
        for _ in 0..5{
            let data = 42;
            println!("produced by 2 {}", data);
            tx.send(data).unwrap();
            // thread::sleep(Duration::from_secs(1));
        }
    });

    let consumer = thread::spawn(move || {
        for _ in 0..10{
            println!("receive {}", rx.recv().unwrap());
        }
    });

    let _ = producer_1.join();
    let _ = producer_2.join();
    let _ = consumer.join();
}