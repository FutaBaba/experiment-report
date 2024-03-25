#![feature(thread_spawn_unchecked)]
#![feature(core_intrinsics)]
#![feature(strict_provenance)]
#![feature(layout_for_ptr)]
#![feature(allocator_api)]
pub mod new_ref {
    use std::rc::Rc;
    use core::ops::Deref;
    use core::fmt;
    use core::ops::DerefMut;
    use std::thread::panicking;
    use std::process::abort;
    use std::mem::ManuallyDrop;

    pub struct RefImmut<T> {
        rc: Rc<T>,
    }

    pub struct RefMut<T> {
        data: T,
    }

    impl<T> RefImmut<T> {
        pub fn clone_immut(&self) -> RefImmut<T> {
            RefImmut { rc: Rc::clone(&self.rc) }
        }

        pub fn into_rc(self) -> Rc<T> {
            self.rc
        }

        pub fn back_to_mut(self) -> RefMut<T> {
            let inner = Rc::into_inner(self.rc);
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
            RefImmut {rc: Rc::new(self.data)}
        }

        pub fn into_inner(self) -> T {
            self.data
        }
    }

    impl<T> Deref for RefImmut<T> {
        type Target = T;
        #[inline]
        fn deref(&self) -> &T {
            &self.rc
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
            let count = Rc::strong_count(&self.rc);
            if count > 1{
                drop(self);
            }
            else if count == 1{
                if panicking() {
                    return
                }
                drop(self);
                panic!("cannot drop");
            }
            else {
                abort();
            }
        }
    }
}