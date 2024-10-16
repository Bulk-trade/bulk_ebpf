use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub struct DummyAllocator;

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = std::alloc::System.alloc(layout);
        println!("Allocated {} bytes at {:?}", layout.size(), ptr);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        println!("Deallocated {:?}", ptr);
        std::alloc::System.dealloc(ptr, layout)
    }
}

// #[global_allocator]
// static ALLOCATOR: DummyAllocator = DummyAllocator;

static TOTAL_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static TOTAL_DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

pub fn print_allocator_stats() {
    println!("Allocator Statistics:");
    println!("Total allocations: {}", TOTAL_ALLOCATIONS.load(Ordering::Relaxed));
    println!("Total deallocations: {}", TOTAL_DEALLOCATIONS.load(Ordering::Relaxed));
}