//! [Proof of Concept] Allocator singletons and parameterized collections on stable
//!
//! # Isn't the `alloc` crate going to be stabilized in 1.36?
//!
//! Yes, and you will be able to use it in libraries and `std` binaries but *not* in `no_std`
//! binaries because `alloc` has a hard dependency on `#[global_allocator]` (stable) and, on
//! `no_std`, `#[global_allocator]` has a hard dependency on `#[alloc_error_handler]` which is
//! unstable and has no stabilization date.
//!
//! # Features
//!
//! - You are not limited to a single global allocator; you can create as many allocators as you
//!   want.
//!
//! - Instantiate collections on any of these allocators.
//!
//! - Works on stable.
//!
//! # Cons
//!
//! - Doesn't integrate with the `alloc` crate. Meaning that we need to re-create that crate from
//! scratch.
//!
//! - Dynamically Sized Types (e.g. `Box<[u8]>` and `Box<dyn Fn()>`) are not supported because
//! [`CoerceUnsized`] and [`Unsize`] are unstable APIs.
//!
//! [`CoerceUnsized`]: https://doc.rust-lang.org/core/ops/trait.CoerceUnsized.html
//! [`Unsize`]: https://doc.rust-lang.org/core/marker/trait.Unsize.html
//!
//! # Example
//!
//! ``` ignore
//! #![no_main]
//! #![no_std]
//!
//! use core::alloc::Layout;
//!
//! use alloc_many::{alloc, oom};
//! use alloc_many_bump::{consts, BumpAlloc}; // NOTE: MSRV = 1.36
//! use alloc_many_collections::Box; // instead of the (still) unstable `alloc` crate
//! use cortex_m_rt::entry;
//! use panic_halt as _; // panic handler
//!
//! // instantiate a bump allocator and bind it to the type `A`
//! #[allocator] // instead of the reserved `#[global_allocator]`
//! static A: BumpAlloc<consts::U128> = BumpAlloc::new();
//!
//! #[entry]
//! fn main() -> ! {
//!     // allocate this value on allocator `A`
//!     let _x: Box<A, _> = Box::new(0);
//!
//!     loop {}
//! }
//!
//! // called when any allocator signals OOM
//! #[oom] // instead of the reserved, unstable `#[alloc_error_handler]`
//! fn oom(_: Layout) -> ! {
//!     loop {}
//! }
//! ```
//!
//! (Yes, a bump pointer allocator is not a really good choice for an allocator. You may want to
//! check out [my TLSF allocator][tlsf])
//!
//! [tlsf]: https://github.com/japaric/tlsf
//!
//! # Minimum Supported Rust Version (MSRV)
//!
//! This crate is guaranteed to compile on stable Rust 1.32 and up. It might compile on older
//! versions but that may change in any new patch release.

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]

#[allow(unused_extern_crates)]
#[cfg(test)]
extern crate self as alloc_many;

use core::alloc::Layout;

pub use alloc_many_macros::{allocator, oom};

/// Singleton version of [`core::alloc::GlobalAlloc`][0]
///
/// [0]: https://doc.rust-lang.org/core/alloc/trait.GlobalAlloc.html
pub unsafe trait Alloc {
    /// Returns a pointer meeting the size and alignment guarantees of `layout`
    ///
    /// Singleton version of [`core::alloc::GlobalAlloc::alloc`][0]
    ///
    /// [0]: https://doc.rust-lang.org/core/alloc/trait.GlobalAlloc.html#tymethod.alloc
    unsafe fn alloc(layout: Layout) -> *mut u8;

    /// Deallocate the memory referenced by `ptr`
    ///
    /// Singleton version of [`core::alloc::GlobalAlloc::dealloc`][0]
    ///
    /// [0]: https://doc.rust-lang.org/core/alloc/trait.GlobalAlloc.html#tymethod.dealloc
    unsafe fn dealloc(ptr: *mut u8, layout: Layout);

    /// Behaves like `alloc`, but also ensures that the contents are set to zero before being
    /// returned.
    ///
    /// Singleton version of [`core::alloc::GlobalAlloc::alloc_zeroed`][0]
    ///
    /// [0]: https://doc.rust-lang.org/core/alloc/trait.GlobalAlloc.html#tymethod.alloc_zeroed
    unsafe fn alloc_zeroed(layout: Layout) -> *mut u8;

    /// Shrink or grow a block of memory to the given `new_size`. The block is described by the
    /// given `ptr` pointer and `layout`.
    ///
    /// Singleton version of [`core::alloc::GlobalAlloc::realloc`][0]
    ///
    /// [0]: https://doc.rust-lang.org/core/alloc/trait.GlobalAlloc.html#tymethod.realloc
    unsafe fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8;
}
