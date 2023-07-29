fn f (x:String) {
    println!("{}", x);
}

fn main() {
    let mut x:String = String::from("Hello");//ここでxのownership発生
    x.push_str("World");
    println!("{}", x);
    {
        let y = x;//ここでyにownership移る
        println!("{}", y);
        //}の直前に解放される
    }
    // println!("{}", x); yにownership移っているのでエラー
}