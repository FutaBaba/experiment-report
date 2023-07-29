fn f (x:String) {
    println!("{}", x);
    //ここでStringは解放される
}

fn main() {
    let mut x:String = String::from("Hello"); //xのownership(解放する義務)がここで発生
    x.push_str("World");
    println!("{}", x);
    f(x);
}