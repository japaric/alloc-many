//! Parameterized collections on stable
//!
//! Rewrite of the standard `alloc` crate that works with `no_std` binaries on stable
//! (`alloc` can't be used on stable `no_std` because `#[alloc_error_handler]` is unstable).
//!
//! # Minimum Supported Rust Version (MSRV)
//!
//! This crate is guaranteed to compile on stable Rust 1.33 and up. It might compile on older
//! versions but that may change in any new patch release.

#![deny(missing_docs)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]
#![no_std]

use core::alloc::Layout;

pub mod boxed;
#[cfg(test)]
mod tests;
mod unique;
pub mod vec;

#[allow(improper_ctypes)]
extern "Rust" {
    fn alloc_many_oom(layout: Layout) -> !;
}
