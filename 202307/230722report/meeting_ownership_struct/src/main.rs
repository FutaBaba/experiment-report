struct Test {
    name: String,
}

fn f (x:String,t: &mut Test) {
    println!("{}", x);
    t.name = x;
}

fn main() {
    let mut x:String = String::from("Hello");// ここでxのownership発生
    x.push_str("World");
    println!("{}", x);
    let mut t = Test {
        name: String::from("foo"),
    };
    t.name = x;// ここでt.nameにownership移る
    //スコープが終われば(}の直前)tが解放される=>このときtの内部のnameも解放される
}
