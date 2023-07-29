use std::mem::ManuallyDrop;
use std::thread;
use std::time::Duration;


fn main() {
    let x:String = String::from("Hello");// ここでxのownership発生

    let mut x_drop = ManuallyDrop::new(x);// ここでxのownershipがManuallyDropの内部のフィールドに移る
    unsafe{
        let mut x_taken_0:String = ManuallyDrop::take(&mut x_drop);
        x_taken_0.push_str("World");
        println!("{}", x_taken_0);
        let mut drop2 = ManuallyDrop::new(x_taken_0);
        let x_taken2:String = ManuallyDrop::take(&mut drop2);
        println!("{}", x_taken2 );
    }
    //x_dropはここで解放されるが、内部のフィールドは解放されない!=>これが抜け道!
}