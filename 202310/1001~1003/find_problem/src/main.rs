#![feature(thread_spawn_unchecked)]
#![feature(core_intrinsics)]
use std::thread;
// use std::sync::Arc;
use std::time::Duration;
use std::thread::JoinHandle;
// use std::thread::ScopedJoinHandle;
use core::ptr::NonNull;
use core::marker::PhantomData;
use core::sync::atomic;
use core::ops::Deref;
use core::fmt;
use core::ops::DerefMut;
// use core::mem;
use core::sync::atomic::Ordering::Relaxed;
use core::intrinsics::abort;
use core::sync::atomic::Ordering::Acquire;

const MAX_REFCOUNT: usize = (isize::MAX) as usize;

unsafe impl<T: ?Sized + Sync + Send> Send for BabaArc<T> {}

unsafe impl<T: ?Sized + Sync + Send> Sync for BabaArc<T> {}

pub struct BabaArc<T: ?Sized> {
    ptr: NonNull<BabaArcInner<T>>,
    phantom: PhantomData<BabaArcInner<T>>,
}

struct BabaArcInner<T: ?Sized> {
    strong: atomic::AtomicUsize,

    // the value usize::MAX acts as a sentinel for temporarily "locking" the
    // ability to upgrade weak pointers or downgrade strong ones; this is used
    // to avoid races in `make_mut` and `get_mut`.
    weak: atomic::AtomicUsize,

    write: atomic::AtomicUsize,

    read: atomic::AtomicUsize,

    data: T,
}

impl<T: ?Sized> BabaArc<T> {
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

    pub fn write_count(this: &Self) -> usize {
        this.inner().write.load(Acquire)
    }

    pub fn read_count(this: &Self) -> usize {
        this.inner().read.load(Acquire)
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

    pub fn clone_immut(&self) -> BabaArc<T> {
        if Self::write_count(&self) != 0 {
            panic!("immutable error")
        }
        else {
            let old_size = self.inner().read.fetch_add(1, Relaxed);
            
            // if old_size > MAX_REFCOUNT {
            //     abort();
            // }

            unsafe { Self::from_inner(self.ptr) }
        }
    }

    fn clone_mut(&self) -> BabaArc<T> {
        if Self::write_count(&self) == 0 && Self::read_count(&self) == 0 {
            self.inner().write.fetch_add(1, Relaxed);
            unsafe { Self::from_inner(self.ptr) }
        } else {
            panic!("Write Error mut")
        }
    }
}

impl<T> BabaArc<T> {
    pub fn new(data: T) -> BabaArc<T> {
        // Start the weak pointer count as 1 which is the weak pointer that's
        // held by all the strong pointers (kinda), see std/rc.rs for more info
        let x: Box<_> = Box::new(BabaArcInner {
            strong: atomic::AtomicUsize::new(1),
            weak: atomic::AtomicUsize::new(1),
            write: atomic::AtomicUsize::new(0),
            read: atomic::AtomicUsize::new(0),
            data,
        });
        unsafe { Self::from_inner(Box::leak(x).into()) }
    }
}

impl<T: ?Sized> Deref for BabaArc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // BabaArc::increment_read_count(&self);
        &self.inner().data
    }
}

impl<T: ?Sized> DerefMut for BabaArc<T> {

    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // BabaArc::increment_write_count(&self);
        &mut self.inner_mut().data
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for BabaArc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized> Clone for BabaArc<T> {
    fn clone(&self) -> BabaArc<T> {
        
        let old_size = self.inner().strong.fetch_add(1, Relaxed);
        
        if old_size > MAX_REFCOUNT {
            abort();
        }
        

        unsafe { Self::from_inner(self.ptr) }
    }
}

impl<T: ?Sized> Drop for BabaArc<T> {
    fn drop(&mut self) {
        if Self::write_count(&self) == 1 {
            Self::decrement_write_count(&self);
        }
        else {
            Self::decrement_read_count(&self);
        }
        // println!("drop");
    }
}

// fn f<T> (t: JoinHandle<T>) {
//     t.join();
// }

fn g (a: &BabaArc<String>) -> Result<JoinHandle<()>, std::io::Error>{
    // let builder = thread::Builder::new();
    // let new_a = a.clone();
    unsafe{
        let builder = thread::Builder::new();
        let t1 = builder.spawn_unchecked(|| {
            thread::sleep(Duration::from_secs(1));
            let new_a = BabaArc::clone_immut(a); //ここで問題が起きているっぽい?
            println!("{}", new_a);
            drop(new_a)
        });
        
       return t1
    }
}

fn main() {

    let builder2 = thread::Builder::new();

    let a = String::from("Hello");

    unsafe {
        let mut new_a = BabaArc::new(a);
        // let check_a = &new_a;

        println!("{}, {}", BabaArc::write_count(&new_a), BabaArc::read_count(&new_a));

        let t1 = g(&new_a);//ここでcount +1
    
        let t2 = builder2.spawn_unchecked(|| {
            {
                let mut mut_a = BabaArc::clone_mut(&new_a);
                mut_a.push_str("bbbb");
            }
            
            let immut_a = BabaArc::clone_immut(&new_a);
            println!("{}", immut_a);
            drop(immut_a);
        }).unwrap().join();

        println!("{}, {}", BabaArc::write_count(&new_a), BabaArc::read_count(&new_a));

    }

    thread::sleep(Duration::from_secs(2));
}