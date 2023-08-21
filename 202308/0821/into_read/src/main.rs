use core::sync::atomic;
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use core::mem;
use std::process::abort;

const MAX_REFCOUNT: usize = (isize::MAX) as usize;

unsafe impl<T: ?Sized + Sync + Send> Send for NewArc<T> {}

macro_rules! acquire {
    ($x:expr) => {
        atomic::fence(Acquire)
    };
}

pub struct NewArc<T: ?Sized> {
    ptr: NonNull<NewArcInner<T>>,
    phantom: PhantomData<NewArcInner<T>>,
}

impl<T: ?Sized> NewArc<T> {
    unsafe fn from_inner(ptr: NonNull<NewArcInner<T>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }
}

pub struct NewWeak<T: ?Sized> {
    // This is a `NonNull` to allow optimizing the size of this type in enums,
    // but it is not necessarily a valid pointer.
    // `Weak::new` sets this to `usize::MAX` so that it doesn’t need
    // to allocate space on the heap. That's not a value a real pointer
    // will ever have because RcBox has alignment at least 2.
    // This is only possible when `T: Sized`; unsized `T` never dangle.
    ptr: NonNull<NewArcInner<T>>,
}

struct NewArcInner<T: ?Sized> {
    strong: atomic::AtomicUsize,

    // the value usize::MAX acts as a sentinel for temporarily "locking" the
    // ability to upgrade weak pointers or downgrade strong ones; this is used
    // to avoid races in `make_mut` and `get_mut`.
    weak: atomic::AtomicUsize,

    data: T,
}
impl<T> NewArc<T>  {
    pub fn new(data: T) -> NewArc<T> {
        // Start the weak pointer count as 1 which is the weak pointer that's
        // held by all the strong pointers (kinda), see std/rc.rs for more info
        let x: Box<_> = Box::new(NewArcInner {
            strong: atomic::AtomicUsize::new(1),
            weak: atomic::AtomicUsize::new(1),
            data,
        });
        unsafe { Self::from_inner(Box::leak(x).into()) }
    }

    pub fn into_inner(this: Self) -> Option<T> {
        // Make sure that the ordinary `Drop` implementation isn’t called as well
        let mut this = mem::ManuallyDrop::new(this);

        // Following the implementation of `drop` and `drop_slow`
        if this.inner().strong.fetch_sub(1, Release) != 1 {
            return None;
        }

        acquire!(this.inner().strong);

        // SAFETY: This mirrors the line
        //
        //     unsafe { ptr::drop_in_place(Self::get_mut_unchecked(self)) };
        //
        // in `drop_slow`. Instead of dropping the value behind the pointer,
        // it is read and eventually returned; `ptr::read` has the same
        // safety conditions as `ptr::drop_in_place`.
        let inner = unsafe { ptr::read(Self::get_mut_unchecked(&mut this)) };

        drop(NewWeak { ptr: this.ptr });

        Some(inner)
    }

    pub fn into_read(this: Self) -> Option<T> {
        // Make sure that the ordinary `Drop` implementation isn’t called as well
        let mut this = mem::ManuallyDrop::new(this);

        // Following the implementation of `drop` and `drop_slow`
        // if this.inner().strong.fetch_sub(1, Release) != 1 {
        //     return None;
        // }
        // this.inner().strong.fetch_sub(1, Release);

        acquire!(this.inner().strong);

        // SAFETY: This mirrors the line
        //
        //     unsafe { ptr::drop_in_place(Self::get_mut_unchecked(self)) };
        //
        // in `drop_slow`. Instead of dropping the value behind the pointer,
        // it is read and eventually returned; `ptr::read` has the same
        // safety conditions as `ptr::drop_in_place`.
        let inner = unsafe { ptr::read(Self::get_mut_unchecked(&mut this)) };

        // drop(NewWeak { ptr: this.ptr });

        Some(inner)
    }

    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
        unsafe { &mut (*this.ptr.as_ptr()).data }
    }
}

impl <T: ?Sized> NewArc<T> {
    fn inner(&self) -> &NewArcInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // contents.
        unsafe { self.ptr.as_ref() }
    }
}
impl<T: ?Sized> Clone for NewArc<T> {
    #[inline]
    fn clone(&self) -> NewArc<T> {
        // Using a relaxed ordering is alright here, as knowledge of the
        // original reference prevents other threads from erroneously deleting
        // the object.
        //
        // As explained in the [Boost documentation][1], Increasing the
        // reference counter can always be done with memory_order_relaxed: New
        // references to an object can only be formed from an existing
        // reference, and passing an existing reference from one thread to
        // another must already provide any required synchronization.
        //
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        
        
        let old_size = self.inner().strong.fetch_add(1, Relaxed);
        

        // However we need to guard against massive refcounts in case someone is `mem::forget`ing
        // Arcs. If we don't do this the count can overflow and users will use-after free. This
        // branch will never be taken in any realistic program. We abort because such a program is
        // incredibly degenerate, and we don't care to support it.
        //
        // This check is not 100% water-proof: we error when the refcount grows beyond `isize::MAX`.
        // But we do that check *after* having done the increment, so there is a chance here that
        // the worst already happened and we actually do overflow the `usize` counter. However, that
        // requires the counter to grow from `isize::MAX` to `usize::MAX` between the increment
        // above and the `abort` below, which seems exceedingly unlikely.
        //
        // This is a global invariant, and also applies when using a compare-exchange loop to increment
        // counters in other methods.
        // Otherwise, the counter could be brought to an almost-overflow using a compare-exchange loop,
        // and then overflow using a few `fetch_add`s.
        if old_size > MAX_REFCOUNT {
            abort();
        }
        

        unsafe { Self::from_inner(self.ptr) }
    }
}

unsafe impl<#[may_dangle] T: ?Sized> Drop for Weak<T> {
    /// Drops the `Weak` pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Weak};
    ///
    /// struct Foo;
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("dropped!");
    ///     }
    /// }
    ///
    /// let foo = Arc::new(Foo);
    /// let weak_foo = Arc::downgrade(&foo);
    /// let other_weak_foo = Weak::clone(&weak_foo);
    ///
    /// drop(weak_foo);   // Doesn't print anything
    /// drop(foo);        // Prints "dropped!"
    ///
    /// assert!(other_weak_foo.upgrade().is_none());
    /// ```
    fn drop(&mut self) {
        // If we find out that we were the last weak pointer, then its time to
        // deallocate the data entirely. See the discussion in Arc::drop() about
        // the memory orderings
        //
        // It's not necessary to check for the locked state here, because the
        // weak count can only be locked if there was precisely one weak ref,
        // meaning that drop could only subsequently run ON that remaining weak
        // ref, which can only happen after the lock is released.
        let inner = if let Some(inner) = self.inner() { inner } else { return };

        if inner.weak.fetch_sub(1, Release) == 1 {
            acquire!(inner.weak);
            unsafe { Global.deallocate(self.ptr.cast(), Layout::for_value_raw(self.ptr.as_ptr())) }
        }
    }
}

fn main () {
    let x = NewArc::new(String::from("Hello"));
    let y = NewArc::clone(&x);
    let z1 = String::from("Hello");
    let z2 = String::from("Hello");

    // Two threads calling `Arc::into_inner` on both clones of an `Arc`:
    let x_thread = std::thread::spawn(|| NewArc::into_read(x));
    let y_thread = std::thread::spawn(|| NewArc::into_read(y));

    let x_inner_value = x_thread.join().unwrap();
    let y_inner_value = y_thread.join().unwrap();

    println!("World");
    assert!(matches!(
        (x_inner_value, y_inner_value),
        (Some(z1), Some(z2))
    ));
    println!("World");
    //println!("{}",y_inner_value);
}