#![feature(thread_spawn_unchecked)]
#![feature(core_intrinsics)]
#![feature(strict_provenance)]
#![feature(layout_for_ptr)]
#![feature(allocator_api)]
pub mod fractional_ref {
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
    use std::mem::forget;
    use std::alloc::alloc;
    use core::marker::PhantomData;

    const MAX_REFCOUNT: usize = (isize::MAX) as usize;
    unsafe impl<T: ?Sized + Sync + Send> Send for FRefImmut<T> {}
    unsafe impl<T: ?Sized + Sync + Send> Send for FRefMut<T> {}
    unsafe impl<T: ?Sized + Sync + Send> Sync for FRefImmut<T> {}
    unsafe impl<T: ?Sized + Sync + Send> Sync for FRefMut<T> {}

    pub struct FRefImmut<T: ?Sized> {
        ptr: NonNull<FRefInner<T>>,
        phantom: PhantomData<FRefInner<T>>
    }

    pub struct FRefMut<T: ?Sized> {
        data: T
    }

    struct FRefInner<T: ?Sized> {
        ref_count: atomic::AtomicUsize,
        data: T
    }

    impl<T: ?Sized> FRefImmut<T> {
        pub fn clone_immut(&self) -> FRefImmut<T> {
            let old_size = self.inner().ref_count.fetch_add(1, Release);
            if old_size > MAX_REFCOUNT {
                abort();
            }
            FRefImmut { ptr: self.ptr, phantom: PhantomData }
        }

        fn inner(&self) -> &FRefInner<T> {
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

    impl<T> FRefImmut<T> {
        pub fn back_to_mut(self) -> FRefMut<T> {
            if self.inner().ref_count.load(Acquire) != 1{
                panic!("count != 1");
            }
            let mut this = ManuallyDrop::new(self);
            let inner = unsafe { ptr::read(&mut (*this.ptr.as_ptr()).data)};
            unsafe {
                dealloc(this.ptr.cast().as_mut(), Layout::for_value_raw(this.ptr.as_ptr()))
            }
            FRefMut { data: inner }
        }
        pub fn back_and_drop(self) {
            drop(self.back_to_mut())
        }
    }

    impl<T> FRefMut<T> {
        pub fn new(data: T) -> FRefMut<T> {
            Self { data: data }
        }

        pub fn get_mut(this: &mut Self) -> &mut T {
            // We are careful to *not* create a reference covering the "count" fields, as
            // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
            &mut *this
        }

        pub fn to_immut(mut self: FRefMut<T>) -> FRefImmut<T> {
            let mut d = ManuallyDrop::new(self.data);
            let inner = unsafe{ptr::read(d.deref_mut())};
            let x = Box::new(FRefInner {
                ref_count: AtomicUsize::new(1),
                data: inner,
            });
            FRefImmut {ptr: Box::leak(x).into(), phantom:PhantomData}
        }
    }

    impl<T> Deref for FRefImmut<T> {
        type Target = T;
        #[inline]
        fn deref(&self) -> &T {
            &self.inner().data
        }
    }

    impl<T> Deref for FRefMut<T> {
        type Target = T;
        #[inline]
        fn deref(&self) -> &T {
            &self.data
        }
    }

    impl<T> DerefMut for FRefMut<T> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.data
        }
    }

    impl<T: fmt::Display> fmt::Display for FRefImmut<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(&**self, f)
        }
    }

    impl<T: fmt::Display> fmt::Display for FRefMut<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(&**self, f)
        }
    }

    impl<T: ?Sized> Drop for FRefImmut<T> {
        fn drop(&mut self) {
            let count = self.inner().ref_count.fetch_sub(1, Relaxed);
            atomic::fence(Acquire);
            if count > 1{
                return
            } else if count == 1{
                unsafe { ptr::drop_in_place(self.ptr.as_ptr()) };
                if !panicking() {
                    panic!("drop without back_to_mut");
                }
            } else {
                abort();
            }
        }
    }
    impl<T: PartialEq> PartialEq for FRefMut<T> {
        #[inline]
        fn eq(&self, other: &FRefMut<T>) -> bool {
            **self == **other
        }
        #[inline]
        fn ne(&self, other: &FRefMut<T>) -> bool {
            **self != **other
        }
    }
    impl<T: PartialEq> PartialEq for FRefImmut<T> {
        #[inline]
        fn eq(&self, other: &FRefImmut<T>) -> bool {
            **self == **other
        }
        #[inline]
        fn ne(&self, other: &FRefImmut<T>) -> bool {
            **self != **other
        }
    }
    impl<T: Eq> Eq for FRefMut<T> {}
    impl<T: Eq> Eq for FRefImmut<T> {}
    impl<T: ?Sized> AsRef<T> for FRefImmut<T> {
        fn as_ref(&self) -> &T {
            &self.inner().data
        }
    }
}