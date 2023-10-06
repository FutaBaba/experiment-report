#![feature(core_intrinsics)]
use core::intrinsics::abort;

fn main() {
    println!("Hello, world!");
    abort();
}
