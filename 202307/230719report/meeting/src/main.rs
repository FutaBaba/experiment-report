struct Test {
    name: String,
}

fn f (x:String,t: &mut Test) {
    println!("{}", x);
    t.name = x;
}

fn main() {
    let mut x:String = String::from("Hello");
    x.push_str("World");
    println!("{}", x);
    let mut t = Test {
        name: String::from("foo"),
    };
    t.name = x;
    // f(x,&mut t);
    // let z = t.name;
    // println!("{}", z);
    // {
    //     let y = x;
    //     println!("{}", y);
    // }
    // println!("{}", x);
}
