use dynamic_example::new_ref::RefImmut;
use dynamic_example::new_ref::RefMut;
fn main() {
    let mut vec: Vec<RefImmut<String>> = Vec::new();
    let mut s = RefMut::new(String::from("Hello"));
    let mut ss = RefMut::new(String::from("World"));
    let s1 = s.to_immut();
    let s2 = s1.clone_immut();
    let s3 = s1.clone_immut();
    let ss1 = ss.to_immut();
    let ss2 = ss1.clone_immut();
    let ss3 = ss1.clone_immut();
    vec.push(s1);
    vec.push(ss1);
    vec.push(s2);
    vec.push(ss2);

    let mut i = 0;
    while  i < vec.len() {
        // 左辺をStringに
        if vec[i] == RefMut::new(String::from("Hello")).to_immut() {
            vec.remove(i);
        }
        i += 1;
    }
    // removehello 関数
    let mut s4 = s3.back_to_mut();
    s4.push_str("World");
    println!("{}, from main", s4);

    for i in 0..2 {
        println!("{}", vec[i]);
    }
    drop(vec);
    let mut ss4 = ss3.back_to_mut();
    ss4.push_str("World");
    println!("{}, from main", ss4);
    // Rustでできる
}
// 配列