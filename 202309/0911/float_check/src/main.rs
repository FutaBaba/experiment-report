#[repr(C)]
union MyUnion {
    f1: u32,
    f2: f32,
}

fn main() {
    let u = MyUnion { f1: 1 };
    let f = unsafe { u.f2 };
    println!("{}",f);
}
