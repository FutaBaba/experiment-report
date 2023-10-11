#![feature(thread_spawn_unchecked)]
#![feature(core_intrinsics)]
use std::thread;
// use std::sync::Weak;
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

const MAX_REFCOUNT: usize = (isize::MAX) as usize;

unsafe impl<T: ?Sized + Sync + Send> Send for BabaArcOrig<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for BabaArcOrig<T> {}
unsafe impl<T: ?Sized + Sync + Send> Send for BabaArcImmut<T> {}
unsafe impl<T: ?Sized + Sync + Send> Send for BabaArcMut<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for BabaArcImmut<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for BabaArcMut<T> {}
use crate::alloc::{AllocError, Allocator, Global, Layout};

pub struct BabaArcOrig<T: ?Sized> {
    ptr: NonNull<BabaArcInner<T>>,
    phantom: PhantomData<BabaArcInner<T>>,
}

pub struct BabaArcImmut<T: ?Sized> {
    ptr: NonNull<BabaArcInner<T>>,
    phantom: PhantomData<BabaArcInner<T>>,
}

pub struct BabaArcMut<T: ?Sized> {
    ptr: NonNull<BabaArcInner<T>>,
    phantom: PhantomData<BabaArcInner<T>>,
}

pub struct BabaWeak<
    T: ?Sized,
    A: Allocator = Global,
> {
    // This is a `NonNull` to allow optimizing the size of this type in enums,
    // but it is not necessarily a valid pointer.
    // `Weak::new` sets this to `usize::MAX` so that it doesn’t need
    // to allocate space on the heap. That's not a value a real pointer
    // will ever have because RcBox has alignment at least 2.
    // This is only possible when `T: Sized`; unsized `T` never dangle.
    ptr: NonNull<BabaArcInner<T>>,
    alloc: A,
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

impl<T: ?Sized> BabaArcOrig<T> {
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

    //RustのArcより引用
    //https://doc.rust-lang.org/src/alloc/sync.rs.html#1746
    unsafe fn drop_slow(&mut self) {
        // Destroy the data at this time, even though we must not free the box
        // allocation itself (there might still be weak pointers lying around).
        unsafe { ptr::drop_in_place(Self::get_mut_unchecked(self)) };

        // Drop the weak ref collectively held by all strong references
        // Take a reference to `self.alloc` instead of cloning because 1. it'll
        // last long enough, and 2. you should be able to drop `Arc`s with
        // unclonable allocators
        drop(BabaWeak { ptr: self.ptr, alloc: &self.alloc });
    }

    //RustのArcより引用
    //https://doc.rust-lang.org/src/alloc/sync.rs.html#2303
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
        unsafe { &mut (*this.ptr.as_ptr()).data }
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

    pub fn clone_immut(&self) -> BabaArcImmut<T> {
        println!("{}, {}, from clone_immut", Self::read_count(&self), Self::write_count(&self));
        if Self::write_count(&self) != 0 {
            panic!("immutable error")
        }
        else {
            let old_size = self.inner().read.fetch_add(1, Relaxed);
            println!("{}, {}, from clone_immut", Self::read_count(&self), Self::write_count(&self));
            
            if old_size > MAX_REFCOUNT {
                abort();
            }

            unsafe { BabaArcImmut::from_inner_immut(self.ptr) }
        }
    }

    fn clone_mut(&self) -> BabaArcMut<T> {
        println!("{}, {}, from clone_mut", Self::read_count(&self), Self::write_count(&self));
        if Self::write_count(&self) == 0 && Self::read_count(&self) == 0 {
            self.inner().write.fetch_add(1, Relaxed);
            println!("{}, {}, from clone_mut", Self::read_count(&self), Self::write_count(&self));
            unsafe { BabaArcMut::from_inner_mut(self.ptr) }
        } else {
            panic!("Write Error mut")
        }
    }
}

impl<T: ?Sized> BabaArcImmut<T> {
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

    pub fn increment_read_count(this: &Self) {
        this.inner().read.fetch_add(1, Relaxed);
    }

    pub fn decrement_read_count(this: &Self) {
        this.inner().read.fetch_sub(1, Relaxed);
    }
}

impl<T: ?Sized> BabaArcMut<T> {
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

//BabaArcOrig自身は(0, 0), BabaArcMut(1, 1), BabaArcImmut(n, 0)

impl<T> BabaArcOrig<T> {
    pub fn new(data: T) -> BabaArcOrig<T> {
        // Start the weak pointer count as 1 which is the weak pointer that's
        // held by all the strong pointers (kinda), see std/rc.rs for more info
        let x: Box<_> = Box::new(BabaArcInner {
            strong: atomic::AtomicUsize::new(1),
            // weak: atomic::AtomicUsize::new(1),
            write: atomic::AtomicUsize::new(0),//自身も含める 
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
        // BabaArc::increment_read_count(&self);
        &self.inner().data
    }
}

impl<T: ?Sized> DerefMut for BabaArcMut<T> {

    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // BabaArc::increment_write_count(&self);
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

impl<T: ?Sized> Clone for BabaArcOrig<T> {
    fn clone(&self) -> BabaArcOrig<T> {
        
        let old_size = self.inner().strong.fetch_add(1, Relaxed);
        
        if old_size > MAX_REFCOUNT {
            abort();
        }
        

        unsafe { Self::from_inner(self.ptr) }
    }
}

//RustのArcのDropより引用
//
impl<T: ?Sized> Drop for BabaArcOrig<T> {
    fn drop(&mut self) {
        unsafe {
            self.drop_slow();
        }
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

fn g (a: &BabaArcOrig<String>) -> Result<JoinHandle<()>, std::io::Error>{
    println!("{}, {}, from g", BabaArcOrig::read_count(a), BabaArcOrig::write_count(a));
    // let builder = thread::Builder::new();
    // let new_a = a.clone();
    unsafe{
        let builder = thread::Builder::new();
        let t1 = builder.spawn_unchecked(|| {
            thread::sleep(Duration::from_secs(1));
            let new_a = BabaArcOrig::clone_immut(a); //ここで問題が起きているっぽい?
            println!("{}, from g", new_a);
            println!("{}, {}, from g", BabaArcOrig::read_count(a), BabaArcOrig::write_count(a));
            drop(new_a)
        });
        
       return t1
    }
}

fn main() {

    let builder2 = thread::Builder::new();

    let a = String::from("Hello");

    // let mut new_a = BabaArcOrig::new(a);

    unsafe {
        let mut new_a = BabaArcOrig::new(a);

        println!("{}, {}, from main", BabaArcOrig::read_count(&new_a), BabaArcOrig::write_count(&new_a));

        let t1 = g(&new_a);//ここでcount +1
    
        let t2 = builder2.spawn_unchecked(|| {
            {
                let mut mut_a = BabaArcOrig::clone_mut(&new_a);
                mut_a.push_str("bbbb");
                println!("{}, {}, from main", BabaArcOrig::read_count(&new_a), BabaArcOrig::write_count(&new_a));
            }
            println!("{}, {}, from main", BabaArcOrig::read_count(&new_a), BabaArcOrig::write_count(&new_a));
            let immut_a = BabaArcOrig::clone_immut(&new_a);
            println!("{}, from main", immut_a);
            println!("{}, {}, from main", BabaArcOrig::read_count(&new_a), BabaArcOrig::write_count(&new_a));
            drop(immut_a);
            println!("{}, {}, from main", BabaArcOrig::read_count(&new_a), BabaArcOrig::write_count(&new_a));
        }).unwrap().join();

        println!("{}, {}, from main", BabaArcOrig::read_count(&new_a), BabaArcOrig::write_count(&new_a));
        drop(new_a);
    }

    // println!("{}, {}, from main", BabaArcOrig::read_count(&new_a), BabaArcOrig::write_count(&new_a));
    thread::sleep(Duration::from_secs(2));
    // println!("{}, {}, from main", BabaArcOrig::read_count(&new_a), BabaArcOrig::write_count(&new_a));
}