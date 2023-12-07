use std::thread;

fn my_spawn (a: &String) -> thread::JoinHandle<()>{
    let t = thread::spawn(|| {
        println!("{}", &a);
    });
    return t
}
fn main() {
    let mut a = String::from("Hello");
    let t1 = my_spawn(&a);
    let t2 = my_spawn(&a);
    t1.join();
    t2.join(); // この時点でownershipは返却済みであるはず
    a.push_str("World"); // Rustではエラー
}
    