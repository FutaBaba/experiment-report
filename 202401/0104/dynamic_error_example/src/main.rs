use dynamic_error_example::new_ref::RefImmut;
use dynamic_error_example::new_ref::RefMut;

fn main() {
    let mut h = RefMut::new(String::from("Hello"));
    let h_immut = h.to_immut();
    println!("{}", h_immut);
    // let h_mut = h_immut.back_to_mut();
}