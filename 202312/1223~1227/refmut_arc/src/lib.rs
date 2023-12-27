#![feature(thread_spawn_unchecked)]
#![feature(core_intrinsics)]
#![feature(strict_provenance)]
#![feature(layout_for_ptr)]
#![feature(allocator_api)]
pub mod new_ref {
    use std::sync::Arc;
    use core::ops::Deref;
    use core::fmt;
    use core::ops::DerefMut;
    use std::marker::PhantomData;

    const MAX_REFCOUNT: usize = (isize::MAX) as usize;
    unsafe impl<T: Sync + Send> Send for RefImmut<T> {}
    unsafe impl<T: Sync + Send> Send for RefMut<T> {}
    unsafe impl<T: Sync + Send> Sync for RefImmut<T> {}
    unsafe impl<T: Sync + Send> Sync for RefMut<T> {}

    pub struct RefImmut<T> {
        arc: Arc<T>,
    }

    pub struct RefMut<T> {
        data: T,
    }

    impl<T: std::cmp::PartialEq> RefImmut<T> {
        pub fn clone_immut(&self) -> RefImmut<T> {
            RefImmut { arc: Arc::clone(&self.arc) }
        }

        pub fn back_to_mut(self) -> RefMut<T> {
            let another = Arc::clone(&self.arc);
            let ptr = Arc::into_raw(another);
            unsafe {
                Arc::decrement_strong_count(ptr);
                let new_another = Arc::from_raw(ptr);
                let inner = Arc::into_inner(new_another);
                if inner == None {
                    panic!("cannot back to mut");
                }
                RefMut { data: inner.unwrap() }
            }
        }
    }

    impl<T> RefMut<T> {
        pub fn new(data: T) -> RefMut<T> {
            Self { data: data }
        }

        pub fn to_immut(self) -> RefImmut<T> {
            RefImmut {arc: Arc::new(self.data)}
        }
    }

    impl<T> Deref for RefImmut<T> {
        type Target = T;
        #[inline]
        fn deref(&self) -> &T {
            &self.arc
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
            drop(&mut self.arc)
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