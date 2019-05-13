//! A pointer type for heap allocations
use core::{alloc::Layout, cmp, fmt, marker::PhantomData, ops, ptr};

use alloc_many::{Alloc, Main};

use crate::unique::Unique;

/// A pointer type for heap allocations
pub struct Box<T, A = Main>
where
    A: Alloc,
    T: ?Sized,
{
    _allocator: PhantomData<A>,
    ptr: Unique<T>,
}

impl<A, T> Box<T, A>
where
    A: Alloc,
{
    /// Allocates memory on the allocator `A` and then places `x` into it.
    pub fn new(value: T) -> Self {
        let layout = Layout::new::<T>();

        unsafe {
            Unique::new(A::alloc(layout) as *mut T)
                .map(|ptr| {
                    ptr.as_ptr().write(value);

                    Box {
                        _allocator: PhantomData,
                        ptr,
                    }
                })
                .unwrap_or_else(|| crate::alloc_many_oom(layout))
        }
    }
}

impl<A, T> ops::Deref for Box<T, A>
where
    T: ?Sized,
    A: Alloc,
{
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<A, T> ops::DerefMut for Box<T, A>
where
    T: ?Sized,
    A: Alloc,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<A, T> Drop for Box<T, A>
where
    A: Alloc,
    T: ?Sized,
{
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::for_value(self.ptr.as_ref());
            let ptr = self.ptr.as_ptr();
            ptr::drop_in_place(ptr);
            A::dealloc(ptr as *mut u8, layout)
        }
    }
}

impl<A, T> fmt::Debug for Box<T, A>
where
    T: ?Sized + fmt::Debug,
    A: Alloc,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as fmt::Debug>::fmt(self, f)
    }
}

impl<A, T> fmt::Display for Box<T, A>
where
    T: ?Sized + fmt::Display,
    A: Alloc,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as fmt::Display>::fmt(self, f)
    }
}

impl<A, T> Eq for Box<T, A>
where
    T: ?Sized + Eq,
    A: Alloc,
{
}

impl<A, B, T> PartialEq<Box<T, B>> for Box<T, A>
where
    T: ?Sized + PartialEq,
    A: Alloc,
    B: Alloc,
{
    fn eq(&self, other: &Box<T, B>) -> bool {
        <T as PartialEq>::eq(self, other)
    }
}

impl<A, B, T> PartialOrd<Box<T, B>> for Box<T, A>
where
    T: ?Sized + PartialOrd,
    A: Alloc,
    B: Alloc,
{
    fn partial_cmp(&self, other: &Box<T, B>) -> Option<cmp::Ordering> {
        <T as PartialOrd>::partial_cmp(self, other)
    }
}
