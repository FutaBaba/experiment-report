use ppl_demo1::fractional_ref::FRefImmut;
use ppl_demo1::fractional_ref::FRefMut;

fn main() {
    let mut vec: Vec<FRefImmut<String>> = Vec::new();
    {
        let h = FRefMut::new(String::from("Hello")).to_immut(); // 参照の生成
        for _ in 0..10 {
            vec.push(h.clone_immut()); 
        }
        vec.retain(|x| *x.as_ref() != String::from("Hello"));
        h.back_to_mut();
    } // 参照カウントが1であることを確認してからdrop
    // vecを用いる処理（hやw自体は用いない）
}  