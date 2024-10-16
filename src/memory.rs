use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct BulkBookAllocator;

static TOTAL_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static TOTAL_DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for BulkBookAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            TOTAL_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        TOTAL_DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    }
}

#[global_allocator]
static ALLOCATOR: BulkBookAllocator = BulkBookAllocator;

pub fn print_allocator_stats() {
    println!("Allocator Statistics:");
    println!("Total allocations: {}", TOTAL_ALLOCATIONS.load(Ordering::Relaxed));
    println!("Total deallocations: {}", TOTAL_DEALLOCATIONS.load(Ordering::Relaxed));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator() {
        let layout = Layout::from_size_align(100, 8).unwrap();
        let ptr = unsafe { ALLOCATOR.alloc(layout) };
        assert!(!ptr.is_null());
        
        unsafe { ALLOCATOR.dealloc(ptr, layout) };
        
        print_allocator_stats();
        
        assert!(TOTAL_ALLOCATIONS.load(Ordering::Relaxed) > 0);
        assert!(TOTAL_DEALLOCATIONS.load(Ordering::Relaxed) > 0);
    }
}