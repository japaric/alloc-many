use core::{marker::PhantomData, mem, ptr::NonNull};

#[allow(explicit_outlives_requirements)] // false positive?
pub struct Unique<T>
where
    T: ?Sized,
{
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T> Unique<T>
where
    T: ?Sized,
{
    pub const fn empty() -> Self
    where
        T: Sized,
    {
        unsafe { Self::new_unchecked(mem::align_of::<T>() as *mut T) }
    }

    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr),
            _marker: PhantomData,
        }
    }

    pub fn new(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self {
            ptr,
            _marker: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub unsafe fn as_ref(&self) -> &T {
        self.ptr.as_ref()
    }

    pub unsafe fn as_mut(&mut self) -> &mut T {
        self.ptr.as_mut()
    }
}

unsafe impl<T> Send for Unique<T> where T: Send + ?Sized {}

unsafe impl<T> Sync for Unique<T> where T: Sync + ?Sized {}
