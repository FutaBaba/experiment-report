use refmut_arc::new_ref::RefImmut;
use refmut_arc::new_ref::RefMut;

fn main() {
    let s = String::from("Hello");
    let ref_mut = RefMut::new(s);
    let ref_immut1 = ref_mut.to_immut();
    let ref_immut2 = ref_immut1.clone_immut();
    drop(ref_immut2);
    // let new_mut = ref_immut1.back_to_mut();
    // println!("{}", new_mut);
}