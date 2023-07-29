use std::mem::ManuallyDrop;
use std::thread;
use std::time::Duration;


fn main() {
    let x:String = String::from("Hello");


    let mut x_drop = ManuallyDrop::new(x);
    unsafe{
        let mut x_taken_0:String = ManuallyDrop::take(&mut x_drop);
        x_taken_0.push_str("World");
        println!("{}", x_taken_0);
        let mut drop2 = ManuallyDrop::new(x_taken_0);

        
    // }
    // unsafe{
        let x_taken2:String = ManuallyDrop::take(&mut drop2);
        println!("{}", x_taken2 );
    }
       

    // unsafe{
    //     let mut x_taken:String = ManuallyDrop::take(&mut x_drop);
    //     let mut y_taken:String = ManuallyDrop::take(&mut x_drop);

    //     let x_thread = std::thread::spawn(move || {thread::sleep(Duration::from_secs(3));println!("{}", x_taken); println!("World1"); ManuallyDrop::new(x_taken)});
    //     let y_thread = std::thread::spawn(move || {y_taken.push_str("y");println!("{}", y_taken); println!("World2"); ManuallyDrop::new(String::from("a"))});
       
    //     let mut x_join = x_thread.join().unwrap();
    //     let mut y_join = y_thread.join().unwrap();

    //     let x_taken2:String = ManuallyDrop::take(&mut x_drop);
    //     // let y_taken2:String = ManuallyDrop::take(&mut y_join);

    //     println!("{}", x_taken2);
    // }
}