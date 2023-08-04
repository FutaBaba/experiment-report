extern crate crossbeam_channel;

// use crossbeam_channel;
use std::thread;
use std::time::Duration;

fn s () -> crossbeam_channel::Receiver<String>{
    let (tx, rx) = crossbeam_channel::bounded(10);

    let tx_1 = tx.clone();
    let producer_1 = thread::spawn(move || {
        for _ in 0..5{
            let data = String::from("Hello1");
            println!("produced by 1 {}", data);
            tx_1.send(data).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    let producer_2 = thread::spawn(move || {
        for _ in 0..5{
            let data = String::from("Hello2");
            println!("produced by 2 {}", data);
            tx.send(data).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    return rx;
}

fn main() {
    let rx = s();

    let rx_1 = rx.clone();
    let consumer_1 = thread::spawn(move || {
        for _ in 0..5{
            println!("1 receive {}", rx_1.recv().unwrap());
        }
    });

    let consumer_2 = thread::spawn(move || {
        for _ in 0..5{
            println!("2 receive {}", rx.recv().unwrap());
        }
    });

    // let _ = producer_1.join();
    // let _ = producer_2.join();
    let _ = consumer_1.join();
    let _ = consumer_2.join();
}