#![feature(thread_spawn_unchecked)]
#![feature(core_intrinsics)]
use std::thread;
// use std::sync::Arc;
use std::time::Duration;
use std::thread::JoinHandle;
// use std::thread::ScopedJoinHandle;
use core::ptr::{self, NonNull};
use core::marker::PhantomData;
use core::sync::atomic;
use core::ops::Deref;
use core::fmt;
use core::ops::DerefMut;
// use core::mem;
use core::sync::atomic::Ordering::Relaxed;
use core::intrinsics::abort;
use core::sync::atomic::Ordering::Acquire;
use core::sync::atomic::Ordering::Release;

const MAX_REFCOUNT: usize = (isize::MAX) as usize;
unsafe impl<T: ?Sized + Sync + Send> Send for BabaArcImmut<T> {}
unsafe impl<T: ?Sized + Sync + Send> Send for BabaArcMut<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for BabaArcImmut<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for BabaArcMut<T> {}

pub struct BabaArcImmut<T: ?Sized> {
    ptr: NonNull<BabaArcInner<T>>,
    phantom: PhantomData<BabaArcInner<T>>,
}

pub struct BabaArcMut<T: ?Sized> {
    ptr: NonNull<BabaArcInner<T>>,
    phantom: PhantomData<BabaArcInner<T>>,
}

struct BabaArcInner<T: ?Sized> {
    strong: atomic::AtomicUsize,

    // the value usize::MAX acts as a sentinel for temporarily "locking" the
    // ability to upgrade weak pointers or downgrade strong ones; this is used
    // to avoid races in `make_mut` and `get_mut`.
    // weak: atomic::AtomicUsize,

    write: atomic::AtomicUsize,

    read: atomic::AtomicUsize,

    data: T,
}

impl<T: ?Sized> BabaArcImmut<T> {
    pub fn clone_immut(&self) -> BabaArcImmut<T> {
        if Self::strong_count(&self) != 1 {
            panic!("already dropped");
        }
        let old_size = self.inner().read.fetch_add(1, Relaxed);
        if old_size > MAX_REFCOUNT {
            abort();
        }
        unsafe { BabaArcImmut::from_inner_immut(self.ptr) }
    }
    unsafe fn from_inner_immut(ptr: NonNull<BabaArcInner<T>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }

    fn inner(&self) -> &BabaArcInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // contents.
        unsafe { self.ptr.as_ref() }
    }

    pub fn write_count(this: &Self) -> usize {
        this.inner().write.load(Acquire)
    }

    pub fn read_count(this: &Self) -> usize {
        this.inner().read.load(Acquire)
    }

    pub fn strong_count(this: &Self) -> usize {
        this.inner().strong.load(Acquire)
    }

    pub fn increment_read_count(this: &Self) {
        this.inner().read.fetch_add(1, Relaxed);
    }

    pub fn decrement_read_count(this: &Self) {
        this.inner().read.fetch_sub(1, Relaxed);
    }
}

impl<T: ?Sized> BabaArcMut<T> {
    pub fn clone_immut(self) -> BabaArcImmut<T> {
        if Self::strong_count(&self) != 1 {
            panic!("already dropped");
        }
        let old_size = self.inner().read.fetch_add(1, Relaxed);
        if old_size > MAX_REFCOUNT {
            abort();
        }
        unsafe { BabaArcImmut::from_inner_immut(self.ptr) }
    }

    unsafe fn from_inner(ptr: NonNull<BabaArcInner<T>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }

    unsafe fn from_inner_mut(ptr: NonNull<BabaArcInner<T>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }

    fn inner(&self) -> &BabaArcInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // contents.
        unsafe { self.ptr.as_ref() }
    }

    fn inner_mut(&mut self) -> &mut BabaArcInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // contents.
        unsafe { self.ptr.as_mut() }
    }

    pub fn write_count(this: &Self) -> usize {
        this.inner().write.load(Acquire)
    }

    pub fn read_count(this: &Self) -> usize {
        this.inner().read.load(Acquire)
    }

    pub fn strong_count(this: &Self) -> usize {
        this.inner().strong.load(Acquire)
    }

    pub fn increment_write_count(this: &Self) {
        this.inner().write.fetch_add(1, Relaxed);
    }

    pub fn decrement_write_count(this: &Self) {
        this.inner().write.fetch_sub(1, Relaxed);
    }

    pub fn increment_read_count(this: &Self) {
        this.inner().read.fetch_add(1, Relaxed);
    }

    pub fn decrement_read_count(this: &Self) {
        this.inner().read.fetch_sub(1, Relaxed);
    }
}

impl<T> BabaArcMut<T> {
    pub fn new(data: T) -> BabaArcMut<T> {
        // Start the weak pointer count as 1 which is the weak pointer that's
        // held by all the strong pointers (kinda), see std/rc.rs for more info
        let x: Box<_> = Box::new(BabaArcInner {
            strong: atomic::AtomicUsize::new(1),
            // weak: atomic::AtomicUsize::new(1),
            write: atomic::AtomicUsize::new(0),
            read: atomic::AtomicUsize::new(0),
            data,
        });
        unsafe { Self::from_inner(Box::leak(x).into()) }
    }
}

impl<T: ?Sized> Deref for BabaArcImmut<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // BabaArc::increment_read_count(&self);
        &self.inner().data
    }
}

impl<T: ?Sized> Deref for BabaArcMut<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

impl<T: ?Sized> DerefMut for BabaArcMut<T> {

    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner_mut().data
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for BabaArcImmut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for BabaArcMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized> Drop for BabaArcImmut<T> {
    fn drop(&mut self) {
        Self::decrement_read_count(&self);
    }
}

impl<T: ?Sized> Drop for BabaArcMut<T> {
    fn drop(&mut self) {
        Self::decrement_write_count(&self);
    }
}

// fn g (a: &BabaArcMut<String>) -> Result<JoinHandle<()>, std::io::Error>{
//     println!("{}, {}, from g", BabaArcMut::read_count(a), BabaArcMut::write_count(a));
//     // let builder = thread::Builder::new();
//     // let new_a = a.clone();
//     unsafe{
//         let builder = thread::Builder::new();
//         let t1 = builder.spawn_unchecked(|| {
//             thread::sleep(Duration::from_secs(1));
//             let new_a = BabaArcMut::clone_immut(a);
//             println!("{}, from g", new_a);
//             println!("{}, {}, from g", BabaArcMut::read_count(a), BabaArcMut::write_count(a));
//             drop(new_a)
//         });
        
//        return t1
//     }
// }

// fn main() {

//     let builder2 = thread::Builder::new();

//     let a = String::from("Hello");

//     unsafe {
//         let mut new_a = BabaArcMut::new(a);

//         // println!("{}, {}, from main", BabaArcMut::read_count(&new_a), BabaArcMut::write_count(&new_a));

//         let t1 = g(&new_a);//ここでcount +1

//         println!("{}", new_a);
//         t1.unwrap().join();
    
//         let t2 = builder2.spawn_unchecked(|| {
//             new_a.push_str("bbbb");
//             println!("{}, {}, from main", BabaArcMut::read_count(&new_a), BabaArcMut::write_count(&new_a));
//             println!("{}, {}, from main", BabaArcMut::read_count(&new_a), BabaArcMut::write_count(&new_a));
//             let immut_a = BabaArcMut::clone_immut(&new_a);
//             println!("{}, from main", immut_a);
//             println!("{}, {}, from main", BabaArcMut::read_count(&new_a), BabaArcMut::write_count(&new_a));
//             drop(immut_a);
//         }).unwrap().join();

//         println!("{}, {}, from main", BabaArcMut::read_count(&new_a), BabaArcMut::write_count(&new_a));

//     }

//     // println!("{}, {}, from main", BabaArcOrig::read_count(&new_a), BabaArcOrig::write_count(&new_a));
//     thread::sleep(Duration::from_secs(2));
//     // println!("{}, {}, from main", BabaArcOrig::read_count(&new_a), BabaArcOrig::write_count(&new_a));
// }

fn main() {
    let a = String::from("Hello");
    let builder1 = thread::Builder::new();
    let builder2 = thread::Builder::new();
    let mut new_a = BabaArcMut::new(a);
    new_a.push_str("aaaa");

    unsafe {
        let a_1 = BabaArcMut::clone_immut(new_a);
        let a_2 = BabaArcImmut::clone_immut(&a_1);
        let t1 = builder1.spawn_unchecked(|| {
            println!("{}",a_1);
        }).unwrap().join();

        let t2 = builder2.spawn_unchecked(|| {
            println!("{}",a_2);
        }).unwrap().join();
        // new_a.push_str("aaaa");
        // println!("{}", new_a);

    }

}