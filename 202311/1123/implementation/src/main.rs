#![feature(thread_spawn_unchecked)]
#![feature(core_intrinsics)]
#![feature(strict_provenance)]
// use std::marker::PhantomData;
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
// use std::mem::ManuallyDrop;
use core::sync::atomic::AtomicUsize;

const MAX_REFCOUNT: usize = (isize::MAX) as usize;
unsafe impl<T: Sync + Send> Send for ArcImmut<T> {}
unsafe impl<T: Sync + Send> Send for ArcMut<T> {}
unsafe impl<T: Sync + Send> Sync for ArcImmut<T> {}
unsafe impl<T: Sync + Send> Sync for ArcMut<T> {}

pub struct ArcImmut<T> {
    ptr: NonNull<NewArcInner<T>>
}

pub struct ArcMut<T> {
    data: T,
}

struct NewArcInner<T: ?Sized> {
    ref_count: atomic::AtomicUsize,
    data: T
}

impl<T> ArcImmut<T> {
    pub fn clone_immut(&self) -> ArcImmut<T> {
        let old_size = self.inner().ref_count.fetch_add(1, Release);
        if old_size > MAX_REFCOUNT {
            abort();
        }
        ArcImmut { ptr: self.ptr }
    }

    fn inner(&self) -> &NewArcInner<T> {
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

    pub fn back_to_mut(this: Self) -> ArcMut<T> {
        if this.inner().ref_count.load(Acquire) != 1{
            panic!("cannot back to mut");
        }
        let mut this = ManuallyDrop::new(this);
        let inner = unsafe { ptr::read(Self::get_mut_unchecked(&mut this))};
        ArcMut { data: inner }
    }
}

impl<T> ArcMut<T> {
    pub fn new(data: T) -> ArcMut<T> {
        Self { data: data }
    }

    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
        &mut *this
    }

    pub fn clone_immut(self) -> ArcImmut<T> {
        // ToDo test
        let mut this = ManuallyDrop::new(self);
        let inner = unsafe{ptr::read(Self::get_mut_unchecked(&mut this))};
        println!("Hello");
        // ArcImmut{ptr: NonNull::from(&NewArcInner{ref_count: AtomicUsize::new(1), data: inner})}
        let x: Box<_> = Box::new(NewArcInner {
            ref_count: AtomicUsize::new(1),
            data: inner,
        });
        unsafe {ArcImmut {ptr: Box::leak(x).into()} }
    }
}

impl<T> Deref for ArcImmut<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

impl<T> Deref for ArcMut<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T> DerefMut for ArcMut<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: fmt::Display> fmt::Display for ArcImmut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for ArcMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T> Drop for ArcImmut<T> {
    fn drop(&mut self) {
        if self.inner().ref_count.fetch_sub(1, Relaxed) == 1{
            panic!("cannot drop")
        }
        // ToDo 1未満だとエラー
        // count0になったら=>error
    }
}

impl<T> Drop for ArcMut<T> {
    fn drop(&mut self) {
        // let this = ManuallyDrop::new(&mut self.data);
        //ToDo else => panic
        //Mutであるとき、strongは
    }
}

// fn g (a: &ArcImmut<String>) -> Result<JoinHandle<()>, std::io::Error>{
//     unsafe{
//         let builder = thread::Builder::new();
//         let t1 = builder.spawn_unchecked(|| {
//             thread::sleep(Duration::from_secs(1));
//             println!("{}, from g", a);//0.5でprint
//         });       
//        return t1
//     }
// }

// fn main() {
//     let builder2 = thread::Builder::new();

//     let a = String::from("Hello");

//     unsafe {
//         let mut new_a = ArcMut::new(a);//1
//         let immut_a_1 = ArcMut::clone_immut(new_a);//immutableに、countは1
//         let immut_a_2 = ArcImmut::clone_immut(&immut_a_1);//countは2に
//         let t1 = g(&immut_a_1);//2のやつを渡す

//         println!("{}, from main thread", immut_a_2);//2でprint

//         println!("Hello");

//         t1.unwrap().join();

//         drop(immut_a_2);//1に戻す
//         let mut new_a = ArcImmut::back_to_mut(immut_a_1);//mutに戻す
    
//         let _t2 =builder2.spawn_unchecked(|| {
//             new_a.push_str("bbbb");
//             let immut_a = ArcMut::clone_immut(new_a);
//             println!("{}, from main", immut_a);
//             let mut mut_a = ArcImmut::back_to_mut(immut_a);
//         }).unwrap().join();
//    }
//     thread::sleep(Duration::from_secs(2));
// }

fn main() {
    let builder1 = thread::Builder::new();
    let builder2 = thread::Builder::new();

    let a = String::from("Hello");

    unsafe {
        let mut new_a = ArcMut::new(a);//1
        let immut_a1 = ArcMut::clone_immut(new_a);//immutableに、countは1
        println!("{}", ArcImmut::strong_count(&immut_a1));
        let immut_a2 = ArcImmut::clone_immut(&immut_a1);//countは2に
        println!("{}, {}", ArcImmut::strong_count(&immut_a1), ArcImmut::strong_count(&immut_a2));

        let t1 =builder1.spawn_unchecked(|| {
            println!("{}, from t1", immut_a1);
        });
    
        let t2 =builder2.spawn_unchecked(|| {
            println!("{}, from t2", immut_a2);
        });
        let _ = t1.unwrap().join();
        let _ = t2.unwrap().join();

        drop(immut_a2);
        println!("{}", ArcImmut::strong_count(&immut_a1));
        let mut a3 = ArcImmut::back_to_mut(immut_a1);
        a3.push_str("World");
        println!("{}, from main", a3);
   }
}