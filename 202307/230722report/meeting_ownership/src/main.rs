fn main() {
    let mut x:String = String::from("Hello"); //xのownership(解放する義務)がここで発生
    x.push_str("World");
    println!("{}", x);
    //ここ(}の直前)で解放される
}