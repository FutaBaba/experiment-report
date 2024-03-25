use ppl_demo3::fractional_ref::FRefImmut;
use ppl_demo3::fractional_ref::FRefMut;

fn main() {
    let mut vec: Vec<FRefImmut<String>> = Vec::new();
    {
        let h = FRefMut::new(String::from("Hello")).to_immut(); // 参照の生成
        for _ in 0..10 {
            vec.push(h.clone_immut());
        }
        vec.retain(|x| *x.as_ref() != String::from("Hell")); // スペルミス
        // h.back_to_mut(); // back_to_mutせずにdropしているので実行時エラー
    }
    // vecを用いる処理（hやw自体は用いない）
}