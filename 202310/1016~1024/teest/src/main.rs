struct A {
    s: String
}

fn f (s: &mut String) -> &mut String {
    println!("{}", s);
    return s
}

fn main() {
    let mut s1 = String::from("Hello");
    let mut s2 = f(&mut s1);
    println!("{}, {}", s1, s2);
}