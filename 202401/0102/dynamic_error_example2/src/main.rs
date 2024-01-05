use dynamic_error_example2::new_ref::RefImmut;
use dynamic_error_example2::new_ref::RefMut;
fn remove_string(vec: &mut Vec<RefImmut<String>>, s: String) {
    vec.retain(|x| **x != s);
}

fn main() {
    let mut vec: Vec<RefImmut<String>> = Vec::new();
    let mut h = RefMut::new(String::from("Hello"));
    let mut w = RefMut::new(String::from("World"));
    let h1 = h.to_immut();
    let h2 = h1.clone_immut();
    let h3 = h1.clone_immut();
    let w1 = w.to_immut();
    let w2 = w1.clone_immut();
    let w3 = w1.clone_immut();
    vec.push(h1);
    vec.push(w1);
    vec.push(h2);
    vec.push(w2);

    remove_string(&mut vec, String::from("Hello"));
    let mut w4 = w3.back_to_mut();
    w4.push_str("hhhhh");
    println!("{}, from main", w4);

    for i in 0..2 {
        println!("{}", vec[i]);
    }
    drop(vec);
    let mut h4 = h3.back_to_mut();
    h4.push_str("World");
    println!("{}, from main", h4);
}