use std::mem::ManuallyDrop;
use std::thread;
use std::time::Duration;


fn main() {
    let x:String = String::from("Hello");


    let mut x_drop = ManuallyDrop::new(x);
    unsafe{
        let mut x_taken:String = ManuallyDrop::take(&mut x_drop);
        let mut y_taken:String = ManuallyDrop::take(&mut x_drop);

        let x_thread = std::thread::spawn(move || {x_taken.push_str("x");println!("{}", x_taken); println!("World1"); ManuallyDrop::new(x_taken)});
        let y_thread = std::thread::spawn(move || {thread::sleep(Duration::from_secs(3));y_taken.push_str("y");println!("{}", y_taken); println!("World2"); ManuallyDrop::new(y_taken)});
       
        let mut x_join = x_thread.join().unwrap();
        let mut y_join = y_thread.join().unwrap();

        let x_taken2:String = ManuallyDrop::take(&mut x_join);
        // let y_taken2:String = ManuallyDrop::take(&mut y_join);

        // ManuallyDrop::drop(&mut x_drop);
        println!("{}", x_taken2);
        // println!("{}", y_taken2);
    }
}
