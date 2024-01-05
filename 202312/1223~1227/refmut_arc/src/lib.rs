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

    pub struct RefImmut<T> {
        arc: Arc<T>,
    }

    pub struct RefMut<T> {
        data: T,
    }

    impl<T> RefImmut<T> {
        pub fn clone_immut(&self) -> RefImmut<T> {
            RefImmut { arc: Arc::clone(&self.arc) }
        }

        // pub fn into_arc(self) -> Arc<T> {
        //     self.arc
        // }

        pub fn back_to_mut(self) -> RefMut<T> {
            let inner = Arc::into_inner(self.arc);
            match inner {
                None => panic!("cannot back to mut"),
                Some(data) => RefMut { data: data }
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

        pub fn into_inner(self) -> T {
            self.data
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

    // drop
}