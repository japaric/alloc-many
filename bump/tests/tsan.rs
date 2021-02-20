use core::{alloc::Layout, time::Duration};
use std::{
    collections::BTreeSet,
    sync::{Arc, Barrier},
    thread,
};

use alloc_many::{allocator, oom};
use alloc_many_bump::{consts, BumpAlloc};
use alloc_many_collections::boxed::Box;
use threadpool::ThreadPool;

#[oom]
fn oom(_: Layout) -> ! {
    panic!("OOM")
}

#[test]
fn race() {
    #[allocator]
    static A: BumpAlloc<consts::U4096> = BumpAlloc::new();

    const N: usize = 10;
    let (s, r) = crossbeam_channel::bounded(N);
    let pool = ThreadPool::new(N);
    let barrier = Arc::new(Barrier::new(N + 1));
    for _ in 0..N {
        let barrier = barrier.clone();
        let s = s.clone();

        pool.execute(move || {
            // all threads should start allocating at around the same time
            barrier.wait();

            let boxes = (0..100)
                .map(|_| {
                    let mut x = Box::<_, A>::new(0);
                    *x += 1;
                    &*x as *const i32 as usize
                })
                .collect::<BTreeSet<_>>();

            s.send(boxes).unwrap();
        })
    }

    thread::sleep(Duration::from_millis(100));
    barrier.wait();

    // check that there's no aliasing among the boxes
    let mut seen = BTreeSet::new();
    for _ in 0..N {
        let boxes = r.recv().unwrap();

        assert_eq!(seen.intersection(&boxes).count(), 0);
        seen.extend(boxes);
    }
}
