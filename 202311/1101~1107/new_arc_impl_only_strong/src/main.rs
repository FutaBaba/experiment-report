#![feature(thread_spawn_unchecked)]
#![feature(core_intrinsics)]
#![feature(strict_provenance)]
use std::thread;
use std::time::Duration;
use std::thread::JoinHandle;
use core::ptr::{self, NonNull};
use core::marker::PhantomData;
use core::sync::atomic;
use core::ops::Deref;
use core::fmt;
use core::ops::DerefMut;
use core::sync::atomic::Ordering::Relaxed;
use core::intrinsics::abort;
use core::sync::atomic::Ordering::Acquire;
use core::sync::atomic::Ordering::Release;
use std::mem::ManuallyDrop;

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

    data: T,
}

impl<T: ?Sized> BabaArcImmut<T> {
    pub fn clone_immut(&self) -> BabaArcImmut<T> {
        let old_size = self.inner().strong.fetch_add(1, Relaxed);
        if old_size > MAX_REFCOUNT {
            abort();
        }
        unsafe { BabaArcImmut::from_inner(self.ptr) }
    }

    unsafe fn from_inner(ptr: NonNull<BabaArcInner<T>>) -> Self {
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

    pub fn strong_count(this: &Self) -> usize {
        this.inner().strong.load(Acquire)
    }

    pub fn decrement_strong_count(this: &Self) {
        this.inner().strong.fetch_sub(1, Relaxed);
    }

    pub fn gather(this: &Self, other: Self) {
        //panic() さしている先は同じであるか
        // this.inner().strong.fetch_sub(1, Relaxed);
        // dropでcount減らしている
        if this.ptr.addr() != other.ptr.addr() {
            panic!("different reference");
        }
    }

    pub fn back_to_mut(this: Self) -> BabaArcMut<T> {
        if Self::strong_count(&this) != 1{
            panic!("cannot back to mut");
        }
        this.inner().strong.fetch_add(1, Relaxed);
        unsafe { BabaArcMut::from_inner(this.ptr) }
    }

    unsafe fn drop_slow(&mut self) {
        // Destroy the data at this time, even though we must not free the box
        // allocation itself (there might still be weak pointers lying around).
        unsafe { ptr::drop_in_place(Self::get_mut_unchecked(self)) };

        // Drop the weak ref collectively held by all strong references
        // Take a reference to `self.alloc` instead of cloning because 1. it'll
        // last long enough, and 2. you should be able to drop `Arc`s with
        // unclonable allocators
        // drop(Weak { ptr: self.ptr, alloc: &self.alloc });
    }

    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
        unsafe { &mut (*this.ptr.as_ptr()).data }
    }
}

impl<T: ?Sized> BabaArcMut<T> {
    pub fn clone_immut(self) -> BabaArcImmut<T> {
        let this = ManuallyDrop::new(self.ptr);
        unsafe { BabaArcImmut::from_inner(*this) }
    }

    unsafe fn from_inner(ptr: NonNull<BabaArcInner<T>>) -> Self {
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

    pub fn strong_count(this: &Self) -> usize {
        this.inner().strong.load(Acquire)
    }

    pub fn decrement_strong_count(this: &Self) {
        this.inner().strong.fetch_sub(1, Relaxed);
    }

    unsafe fn drop_slow(&mut self) {
        // Destroy the data at this time, even though we must not free the box
        // allocation itself (there might still be weak pointers lying around).
        unsafe { ptr::drop_in_place(Self::get_mut_unchecked(self)) };

        // Drop the weak ref collectively held by all strong references
        // Take a reference to `self.alloc` instead of cloning because 1. it'll
        // last long enough, and 2. you should be able to drop `Arc`s with
        // unclonable allocators
        // drop(Weak { ptr: self.ptr, alloc: &self.alloc });
    }

    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
        unsafe { &mut (*this.ptr.as_ptr()).data }
    }
}

impl<T> BabaArcMut<T> {
    pub fn new(data: T) -> BabaArcMut<T> {
        // Start the weak pointer count as 1 which is the weak pointer that's
        // held by all the strong pointers (kinda), see std/rc.rs for more info
        let x: Box<_> = Box::new(BabaArcInner {
            strong: atomic::AtomicUsize::new(1),
            data,
        });
        unsafe { Self::from_inner(Box::leak(x).into()) }
    }
}

impl<T: ?Sized> Deref for BabaArcImmut<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
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
        if self.inner().strong.fetch_sub(1, Relaxed) == 1{
            unsafe { self.drop_slow(); }
        }
        // Self::decrement_strong_count(&self);
        // count0になったら=>error
    }
}

impl<T: ?Sized> Drop for BabaArcMut<T> {
    fn drop(&mut self) {
        // Self::decrement_strong_count(&self);
        // To Do strongのcountを確認してから開放
        unsafe { self.drop_slow(); }
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
    let mut new_a = BabaArcMut::new(a);//1
    new_a.push_str("aaaa");

    let a_1 = BabaArcMut::clone_immut(new_a);//1 new_aはdrop
    let a_2 = BabaArcImmut::clone_immut(&a_1);//2
    unsafe {
        let t1 = builder1.spawn_unchecked(|| {
            println!("{}",a_1);
        }).unwrap().join();

        let t2 = builder2.spawn_unchecked(|| {
            println!("{}",a_2);
        }).unwrap().join();
    }
    println!("{}, {}, from main", BabaArcImmut::strong_count(&a_1), BabaArcImmut::strong_count(&a_2));
    BabaArcImmut::gather(&a_1, a_2);//ここで1になる
    println!("{}, from main", BabaArcImmut::strong_count(&a_1));
    let mut a_3 = BabaArcImmut::back_to_mut(a_1);//1だからmutにできる
    a_3.push_str("aaaa");
    println!("{}", a_3);
}