#![feature(thread_spawn_unchecked)]
#![feature(core_intrinsics)]
#![feature(strict_provenance)]
#![feature(layout_for_ptr)]
#![feature(allocator_api)]
pub mod new_ref {
    use std::mem::ManuallyDrop;
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
    use std::thread::panicking;

    const MAX_REFCOUNT: usize = (isize::MAX) as usize;
    unsafe impl<T: ?Sized + Sync + Send> Send for RefImmut<T> {}
    unsafe impl<T: ?Sized + Sync + Send> Send for RefMut<T> {}
    unsafe impl<T: ?Sized + Sync + Send> Sync for RefImmut<T> {}
    unsafe impl<T: ?Sized + Sync + Send> Sync for RefMut<T> {}

    pub struct RefImmut<T: ?Sized> {
        ptr: NonNull<NewRefInner<T>>
    }

    pub struct RefMut<T: ?Sized> {
        data: T,
    }

    struct NewRefInner<T: ?Sized> {
        ref_count: atomic::AtomicUsize,
        data: T
    }

    impl<T: ?Sized> RefImmut<T> {
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
    }

    impl<T> RefImmut<T> {
        pub fn back_to_mut(self) -> RefMut<T> {
            if self.inner().ref_count.load(Acquire) != 1{
                panic!("cannot back to mut");
            }
            let mut this = ManuallyDrop::new(self);
            let inner = unsafe { ptr::read(Self::get_mut_unchecked(&mut this))};
            unsafe {
                dealloc(this.ptr.cast().as_mut(), Layout::for_value_raw(this.ptr.as_ptr()))
            }
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
            RefImmut {ptr: Box::leak(x).into()}
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

    impl<T: ?Sized> Drop for RefImmut<T> {
        fn drop(&mut self) {
            let count = self.inner().ref_count.fetch_sub(1, Relaxed);
            if count > 1{
                return
            }
            else if count == 1{
                if panicking() {
                    return
                }
                unsafe {
                    dealloc(self.ptr.cast().as_mut(), Layout::for_value_raw(self.ptr.as_ptr()))
                }
                panic!("cannot drop");
            }
            else {
                abort();
            }
        }
    }
    impl<T: PartialEq> PartialEq for RefMut<T> {
        #[inline]
        fn eq(&self, other: &RefMut<T>) -> bool {
            **self == **other
        }
        #[inline]
        fn ne(&self, other: &RefMut<T>) -> bool {
            **self != **other
        }
    }
    impl<T: PartialEq> PartialEq for RefImmut<T> {
        #[inline]
        fn eq(&self, other: &RefImmut<T>) -> bool {
            **self == **other
        }
        #[inline]
        fn ne(&self, other: &RefImmut<T>) -> bool {
            **self != **other
        }
    }
    impl<T: Eq> Eq for RefMut<T> {}
    impl<T: Eq> Eq for RefImmut<T> {}
}