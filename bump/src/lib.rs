//! A lock-free bump pointer allocator that never frees memory
//!
//! # Minimum Supported Rust Version (MSRV)
//!
//! This crate is guaranteed to compile on stable Rust 1.36 and up. It might compile on older
//! versions but that may change in any new patch release.

#![deny(missing_docs)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]
#![feature(maybe_uninit)]
#![no_std]

use core::{
    alloc::{GlobalAlloc, Layout},
    convert::TryFrom,
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicU16, Ordering},
};

pub use generic_array::typenum::consts;
use generic_array::{ArrayLength, GenericArray};

/// Lock-free bump pointer allocator
pub type BumpAlloc<N> = BumpAlloc_<GenericArray<u8, N>>;

/// Lock-free bump pointer allocator (stable `const-fn` workaround)
pub struct BumpAlloc_<A> {
    // `u16` ought to be big enough for everyone
    index: AtomicU16,
    memory: MaybeUninit<A>,
}

impl<A> BumpAlloc_<A> {
    /// Creates a bump pointer allocator of capacity `N`
    pub const fn new() -> Self {
        Self {
            index: AtomicU16::new(0),
            memory: MaybeUninit::uninit(),
        }
    }
}

unsafe impl<N> GlobalAlloc for BumpAlloc_<GenericArray<u8, N>>
where
    N: ArrayLength<u8>,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = if let Ok(align) = u16::try_from(layout.align()) {
            align
        } else {
            return ptr::null_mut();
        };
        let size = if let Ok(size) = u16::try_from(layout.size()) {
            size
        } else {
            return ptr::null_mut();
        };
        let len = N::U16;

        // XXX(Ordering) TSAN seems to be happy with `Relaxed` ordering though I'm a bit skeptical
        loop {
            let index = self.index.load(Ordering::Relaxed);

            let res =
                ((self.memory.as_ptr() as usize + usize::from(index)) % usize::from(align)) as u16;
            let start = if res == 0 { index } else { index + align - res };

            if start + size > len {
                break ptr::null_mut();
            } else if self
                .index
                .compare_exchange_weak(index, start + size, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break (self.memory.as_ptr() as *mut u8).add(usize::from(start));
            }
        }
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {}
}
