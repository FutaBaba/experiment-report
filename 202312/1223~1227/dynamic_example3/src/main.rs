use dynamic_example3::new_ref::RefImmut;
use dynamic_example3::new_ref::RefMut;
fn remove_string(vec: &mut Vec<RefImmut<String>>, s: String) {
    let mut i = 0;
    while  i < vec.len() {
        if *vec[i] == s {
            vec.remove(i);
        }
        else {
            i += 1;
        }
    }
}

fn main() {
    let mut vec: Vec<RefImmut<String>> = Vec::new();
    let mut h = RefMut::new(String::from("Hello"));
    let mut w = RefMut::new(String::from("World"));
    let h1 = h.to_immut();
    let h2 = h1.clone_immut();
    let h3 = h1.clone_immut();
    let h4 = h1.clone_immut();
    let h5 = h1.clone_immut();
    vec.push(h1);
    vec.push(h2);
    vec.push(h3);
    vec.push(h4);

    remove_string(&mut vec, String::from("Hello"));
    let mut h6 = h5.back_to_mut();
    h6.push_str("World");
    println!("{}, from main", h6);

    println!("{}", vec.is_empty());
}