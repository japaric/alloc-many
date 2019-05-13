//! A contiguous growable array type with heap-allocated contents, written `Vec<T>`.

use core::{alloc::Layout, cmp, marker::PhantomData, mem, ops, slice};

use alloc_many::Alloc;

use crate::unique::Unique;

/// A contiguous growable array type, written `Vec<T>` but pronounced 'vector'.
pub struct Vec<T, A>
where
    A: Alloc,
{
    cap: usize,
    len: usize,
    ptr: Unique<T>,
    _allocator: PhantomData<A>,
}

impl<A, T> Vec<T, A>
where
    A: Alloc,
{
    /// Constructs a new, empty `Vec<T>`
    pub fn new() -> Self {
        let cap = if mem::size_of::<T>() == 0 {
            usize::max_value()
        } else {
            0
        };

        Self {
            _allocator: PhantomData,
            cap,
            len: 0,
            ptr: Unique::empty(),
        }
    }

    /// Returns the number of elements the vector can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Appends an element to the back of a collection.
    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.reserve(1);
        }

        unsafe {
            self.as_mut_ptr().add(self.len).write(elem);
            self.len += 1;
        }
    }

    /// Removes the last element from a vector and returns it, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.len -= 1;
                Some(self.ptr.as_ptr().add(self.len).read())
            }
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in the given
    /// Vec<T>.
    pub fn reserve(&mut self, additional: usize) {
        if self.cap.wrapping_sub(self.len) >= additional {
            return;
        }

        unsafe {
            let (new_cap, new_layout) = amortized_new_capacity(self.len, additional)
                .and_then(|new_cap| layout_array::<T>(new_cap).map(|layout| (new_cap, layout)))
                .unwrap_or_else(|| capacity_overflow());

            let res = match self.current_layout() {
                None => A::alloc(new_layout),
                Some(layout) => A::realloc(self.ptr.as_ptr() as *mut u8, layout, new_layout.size()),
            };

            self.ptr = if let Some(ptr) = Unique::new(res as *mut T) {
                ptr
            } else {
                crate::alloc_many_oom(new_layout)
            };
            self.cap = new_cap;
        }
    }

    fn current_layout(&self) -> Option<Layout> {
        if self.cap == 0 {
            None
        } else {
            unsafe {
                let align = mem::align_of::<T>();
                let size = mem::size_of::<T>() * self.cap;
                Some(Layout::from_size_align_unchecked(size, align))
            }
        }
    }
}

fn amortized_new_capacity(curr: usize, additional: usize) -> Option<usize> {
    let double_cap = curr.checked_mul(2)?;
    let required_cap = curr.checked_add(additional)?;

    Some(cmp::max(double_cap, required_cap))
}

fn capacity_overflow() -> ! {
    panic!("capacity overflow")
}

// unstable methods of `core::alloc::Layout`
fn layout_array<T>(n: usize) -> Option<Layout> {
    layout_repeat(&Layout::new::<T>(), n).map(|(k, _)| k)
}

fn layout_repeat(layout: &Layout, n: usize) -> Option<(Layout, usize)> {
    let padded_size = layout
        .size()
        .checked_add(padding_needed_for(layout, layout.align()))?;

    let alloc_size = padded_size.checked_mul(n)?;

    unsafe {
        // self.align is already known to be valid and alloc_size has been
        // padded already.
        Some((
            Layout::from_size_align_unchecked(alloc_size, layout.align()),
            padded_size,
        ))
    }
}

fn padding_needed_for(layout: &Layout, align: usize) -> usize {
    let len = layout.size();

    let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);

    len_rounded_up.wrapping_sub(len)
}

impl<A, T> ops::Deref for Vec<T, A>
where
    A: Alloc,
{
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<A, T> ops::DerefMut for Vec<T, A>
where
    A: Alloc,
{
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}
