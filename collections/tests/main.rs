use core::alloc::Layout;

use alloc_many::{main_allocator, oom};
use alloc_many_bump::{consts, BumpAlloc};
use alloc_many_collections::boxed::Box;

#[main_allocator]
static A: BumpAlloc<consts::U128> = BumpAlloc::new();

#[test]
fn main() {
    // allocate on the "main" allocator
    // NOTE that the type of `x` MUST be specified (that's not the case for the real `alloc::Box`,
    // which has no `A` / allocator type parameter)
    let x: Box<_> = Box::new(1i32);
    assert_eq!(*x, 1);
}

#[oom]
fn oom(_: Layout) -> ! {
    panic!("OOM")
}
