use rand::Rng;
use fRef_implementation::fractional_ref::FRefImmut;
use fRef_implementation::fractional_ref::FRefMut;

fn main() {
    let w = FRefMut::new(String::from("World")).to_immut(); 
    {
        let mut vec: Vec<FRefImmut<String>> = Vec::new(); 
        let mut rng = rand::thread_rng();
        { 
            let h = FRefMut::new(String::from("Hello")).to_immut(); 
            for _ in 0..10 { 
                if rng.gen() {
                    vec.push(h.clone_immut()); 
                } else {
                    vec.push(w.clone_immut()); 
                }
            }
            vec.retain(|x| *x.as_ref() != String::from("Hello")); // ユーザによるスペルミス
            h.back_to_mut(); // 参照の複製が残っており、ownershipが1になっていないので実行時エラー
        }
        // vecを用いる処理（hやw自体は用いない）
    } // FRefImmutのままFRefMutに戻さずdrop → 実行時エラー
    w.back_to_mut();
}