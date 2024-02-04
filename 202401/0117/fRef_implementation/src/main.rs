use rand::Rng;
use fRef_implementation::fractional_ref::FRefImmut;
use fRef_implementation::fractional_ref::FRefMut;

fn remove_string(vec: &mut Vec<FRefImmut<String>>, s: String) {
    vec.retain(|x| **x != s);
}

fn main() {
    let mut w = FRefMut::new(String::from("World")).to_immut();
    // let w_immut = w.to_immut();
    {
      let mut vec: Vec<FRefImmut<String>> = Vec::new();
      let mut h = FRefMut::new(String::from("Hello"));
      let mut rng = rand::thread_rng();
      {
        let h_immut = h.to_immut(); 
        
        for _ in 0..10 {        
          if rng.gen() {            
            let h_immut1 = h_immut.clone_immut();
            vec.push(h_immut1);
          }
          else {
            let w_immut1 = w.clone_immut();
            vec.push(w_immut1);
          }
        }
        remove_string(&mut vec, String::from("Hello"));
        h_immut.back_to_mut();
      }
      // hとwは用いず、vecを用いる処理
    }
  w.back_to_mut();
}
  
// fn remove_string(vec: &mut Vec<FRefImmut<String>>, s: String) {
//     vec.retain(|x| **x != s);
// }

// fn main() {
//     let mut vec: Vec<FRefImmut<String>> = Vec::new();
//     let h = FRefMut::new(String::from("Hello"));
//     let w = FRefMut::new(String::from("World"));
//     let mut rng = rand::thread_rng();
//     let h_immut = h.to_immut();
//     let w_immut = w.to_immut();
//     for _ in 0..10 {
//         if rng.gen() {
//             let h_immut1 = h_immut.clone_immut();
//             println!("{}", h_immut1);
//             vec.push(h_immut1);
//         }
//         else {
//             let w_immut1 = w_immut.clone_immut();
//             println!("{}", w_immut1);
//             vec.push(w_immut1);
//         }
//     }

//     remove_string(&mut vec, String::from("Hell"));
//     let mut h_new = h_immut.back_to_mut();
//     h_new.push_str("hoge");
//     println!("{}", h_new);
//     drop(h_new);

//     for x in vec.iter() {
//         println!("{}", x);
//     }
//     drop(vec);
//     let w_new = w_immut.back_to_mut();
//     // w_new.push_str("wwwww");
//     // println!("{}, from main", w_new);
// }