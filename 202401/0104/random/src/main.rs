use random::new_ref::RefImmut;
use random::new_ref::RefMut;
use rand::{Rng, thread_rng};

fn remove_string(vec: &mut Vec<RefImmut<String>>, s: String) {
    vec.retain(|x| **x != s);
}

fn main() {
    let mut vec: Vec<RefImmut<String>> = Vec::new();
    let mut h = RefMut::new(String::from("Hello"));
    let mut w = RefMut::new(String::from("World"));
    let mut rng = rand::thread_rng();
    let i = rng.gen_range(1..=10);
    let h_immut = h.to_immut();
    let w_immut = w.to_immut();
    for _ in 0..i {
        if rng.gen() {
            let h_immut1 = h_immut.clone_immut();
            println!("{}", h_immut1);
            vec.push(h_immut1);
        }
        else {
            let w_immut1 = w_immut.clone_immut();
            println!("{}", w_immut1);
            vec.push(w_immut1);
        }
    }

    remove_string(&mut vec, String::from("Hello"));
    let mut h_new = h_immut.back_to_mut();
    h_new.push_str("hhhhh");
    println!("{}, from main", h_new);

    for x in vec.iter() {
        println!("{}", x);
    }
    drop(vec);
    let mut w_new = w_immut.back_to_mut();
    w_new.push_str("World");
    println!("{}, from main", w_new);
}