fn f (mut x: String) -> String {
    x.push_str("111");
    return x;
}

fn main() {
    let mut x = String::from("Hello");
    f(x);
    println!("{}", x);
}
