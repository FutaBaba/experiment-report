#![feature(thread_spawn_unchecked)]
#![feature(core_intrinsics)]
#![feature(strict_provenance)]
#![feature(layout_for_ptr)]
#![feature(allocator_api)]
use std::mem::ManuallyDrop;
use std::thread;
use std::time::Duration;
use std::thread::JoinHandle;
use core::ptr::{self, NonNull};
use core::sync::atomic;
use core::ops::Deref;
use core::fmt;
use core::ops::DerefMut;
use core::sync::atomic::Ordering::Relaxed;
use core::intrinsics::abort;
use core::sync::atomic::Ordering::Acquire;
use core::sync::atomic::Ordering::Release;
use core::sync::atomic::AtomicUsize;
use std::alloc::Layout;
use std::alloc::dealloc;
use std::thread::Builder;

const MAX_REFCOUNT: usize = (isize::MAX) as usize;
unsafe impl<T: Sync + Send> Send for RefImmut<T> {}
unsafe impl<T: Sync + Send> Send for RefMut<T> {}
unsafe impl<T: Sync + Send> Sync for RefImmut<T> {}
unsafe impl<T: Sync + Send> Sync for RefMut<T> {}

pub struct RefImmut<T> {
    ptr: NonNull<NewRefInner<T>>
}

pub struct RefMut<T> {
    data: T,
}

struct NewRefInner<T: ?Sized> {
    ref_count: atomic::AtomicUsize,
    data: T
}

impl<T> RefImmut<T> {
    pub fn clone_immut(&self) -> RefImmut<T> {
        let old_size = self.inner().ref_count.fetch_add(1, Release);
        if old_size > MAX_REFCOUNT {
            abort();
        }
        RefImmut { ptr: self.ptr }
    }

    fn inner(&self) -> &NewRefInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // contents.
        unsafe { self.ptr.as_ref() }
    }

    pub fn strong_count(this: &Self) -> usize {
        this.inner().ref_count.load(Acquire)
    }
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
        unsafe { &mut (*this.ptr.as_ptr()).data }
    }

    pub fn back_to_mut(self) -> RefMut<T> {
        if self.inner().ref_count.load(Acquire) != 1{
            panic!("cannot back to mut");
        }
        let mut this = ManuallyDrop::new(self);
        let inner = unsafe { ptr::read(Self::get_mut_unchecked(&mut this))};
        RefMut { data: inner }
    }
}

impl<T> RefMut<T> {
    pub fn new(data: T) -> RefMut<T> {
        Self { data: data }
    }

    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
        &mut *this
    }

    pub fn to_immut(self) -> RefImmut<T> {
        // ToDo test
        let mut this = ManuallyDrop::new(self);
        let inner = unsafe{ptr::read(Self::get_mut_unchecked(&mut this))};
        let x: Box<_> = Box::new(NewRefInner {
            ref_count: AtomicUsize::new(1),
            data: inner,
        });
        unsafe {RefImmut {ptr: Box::leak(x).into()} }
    }
}

impl<T> Deref for RefImmut<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

impl<T> Deref for RefMut<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T> DerefMut for RefMut<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: fmt::Display> fmt::Display for RefImmut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for RefMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T> Drop for RefImmut<T> {
    fn drop(&mut self) {
        if self.inner().ref_count.fetch_sub(1, Relaxed) > 1{
        }
        else if self.inner().ref_count.fetch_sub(1, Relaxed) == 1{
            unsafe {
                dealloc(self.ptr.cast().as_mut(), Layout::for_value_raw(self.ptr.as_ptr()))
            }
        }
        // ToDo 1未満だとエラー
        // count0になったら=>error
    }
}

// impl<T> Drop for ArcMut<T> {
//     fn drop(&mut self) {
//         // let this = ManuallyDrop::new(&mut self.data);
//         //ToDo else => panic
//         //Mutであるとき、strongは
//     }
// }

fn main() {
    let mut vec: Vec<RefImmut<String>> = Vec::new();
    let mut s = RefMut::new(String::from("Hello"));
    let mut ss = RefMut::new(String::from("World"));
    let s1 = s.to_immut();
    let s2 = s1.clone_immut();
    let s3 = s1.clone_immut();
    let ss1 = ss.to_immut();
    let ss2 = ss1.clone_immut();
    let ss3 = ss1.clone_immut();
    vec.push(s1);
    vec.push(ss1);
    vec.push(s2);
    vec.push(ss2);
    
    let new_s1 = vec.remove(0);
    drop(new_s1);
    let new_s2 = vec.remove(1);
    drop(new_s2);
    // removehello 関数
    let mut s4 = s3.back_to_mut();
    s4.push_str("World");
    println!("{}, from main", s4);

    for i in 0..2 {
        println!("{}", vec[i]);
    }
    drop(vec);
    let mut ss4 = ss3.back_to_mut();
    ss4.push_str("World");
    println!("{}, from main", ss4);
    // Rustでできる
}
// 配列