use core::{
    alloc::Layout,
    sync::atomic::{AtomicU8, Ordering},
};

use alloc_many::{allocator, oom};
use alloc_many_bump::{consts, BumpAlloc};

use crate::boxed::Box;

#[oom]
fn oom(_: Layout) -> ! {
    panic!()
}

#[test]
fn sanity() {
    #[allocator]
    static A: BumpAlloc<consts::U128> = BumpAlloc::new();

    let x: Box<A, _> = Box::new(0u8);
    assert_eq!(*x, 0);

    // test aligned allocation
    let y: Box<A, _> = Box::new(1);
    assert_eq!(&*y as *const i32 as usize % 4, 0);
    assert_eq!(*y, 1);

    let z: Box<A, _> = Box::new([2, 3]);
    assert_eq!(*z, [2, 3]);

    // test `Drop` implementation
    static X: AtomicU8 = AtomicU8::new(2);
    struct Z;
    impl Drop for Z {
        fn drop(&mut self) {
            X.fetch_sub(1, Ordering::Relaxed);
        }
    }

    let w: Box<A, _> = Box::new([Z, Z]);
    drop(w);
    assert_eq!(X.load(Ordering::Relaxed), 0);

    // OOM
    use alloc_many::Alloc;
    unsafe {
        assert!(A::alloc(Layout::from_size_align(128, 1).unwrap()).is_null());
    }
}
