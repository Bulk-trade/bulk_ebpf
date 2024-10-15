use std::alloc::{GlobalAlloc, Layout};
use std::ptr::NonNull;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const SLAB_SIZES: &[usize] = &[16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

pub struct SlabAllocator {
    slabs: Mutex<Vec<(usize, Slab)>>,
    total_allocations: AtomicUsize,
    total_deallocations: AtomicUsize,
}

struct Slab {
    chunk_size: usize,
    free_list: Option<NonNull<FreeListNode>>,
    total_chunks: usize,
    used_chunks: usize,
}

struct FreeListNode {
    next: Option<NonNull<FreeListNode>>,
}

unsafe impl Send for FreeListNode {}
unsafe impl Sync for FreeListNode {}

unsafe impl Send for Slab {}
unsafe impl Sync for Slab {}

impl SlabAllocator {
    const fn new() -> Self {
        SlabAllocator {
            slabs: Mutex::new(Vec::new()),
            total_allocations: AtomicUsize::new(0),
            total_deallocations: AtomicUsize::new(0),
        }
    }

    fn init_slab(&self, size: usize) {
        let mut slabs = self.slabs.lock().unwrap();
        if !slabs.iter().any(|&(s, _)| s == size) {
            let new_slab = Slab::new(size, 100); // Create 100 chunks initially
            slabs.push((size, new_slab));
        }
    }
}

impl Slab {
    fn new(chunk_size: usize, chunk_count: usize) -> Self {
        let mut memory = vec![0u8; chunk_size * chunk_count];
        let mut free_list = None;

        for i in (0..chunk_count).rev() {
            let ptr = unsafe { memory.as_mut_ptr().add(i * chunk_size) as *mut FreeListNode };
            unsafe {
                (*ptr).next = free_list;
                free_list = Some(NonNull::new_unchecked(ptr));
            }
        }

        Slab {
            chunk_size,
            free_list,
            total_chunks: chunk_count,
            used_chunks: 0,
        }
    }
}

unsafe impl GlobalAlloc for SlabAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(layout.align()).next_power_of_two();
        let size = SLAB_SIZES.iter().find(|&&s| s >= size).copied().unwrap_or(size);

        self.init_slab(size);

        let mut slabs = self.slabs.lock().unwrap();
        let slab_index = slabs.iter().position(|&(s, _)| s == size).unwrap();
        let slab = &mut slabs[slab_index].1;

        let ptr = match slab.free_list.take() {
            Some(node) => {
                slab.free_list = (*node.as_ptr()).next;
                slab.used_chunks += 1;
                node.as_ptr() as *mut u8
            }
            None => {
                // Allocate a new chunk if the slab is full
                let new_chunk = std::alloc::alloc(Layout::from_size_align(size, layout.align()).unwrap());
                if !new_chunk.is_null() {
                    slab.total_chunks += 1;
                    slab.used_chunks += 1;
                }
                new_chunk
            }
        };

        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        println!("Allocated {} bytes at {:?}", size, ptr);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(layout.align()).next_power_of_two();
        let size = SLAB_SIZES.iter().find(|&&s| s >= size).copied().unwrap_or(size);

        let mut slabs = self.slabs.lock().unwrap();
        if let Some(slab_index) = slabs.iter().position(|&(s, _)| s == size) {
            let slab = &mut slabs[slab_index].1;
            let node = NonNull::new_unchecked(ptr as *mut FreeListNode);
            (*node.as_ptr()).next = slab.free_list;
            slab.free_list = Some(node);
            slab.used_chunks -= 1;

            self.total_deallocations.fetch_add(1, Ordering::Relaxed);
            println!("Deallocated {} bytes at {:?}", size, ptr);
        } else {
            // If we don't have a slab for this size, it was probably allocated by the system allocator
            std::alloc::dealloc(ptr, layout);
        }
    }
}

#[global_allocator]
static ALLOCATOR: SlabAllocator = SlabAllocator::new();

pub fn print_allocator_stats() {
    println!("Allocator Statistics:");
    println!("Total allocations: {}", ALLOCATOR.total_allocations.load(Ordering::Relaxed));
    println!("Total deallocations: {}", ALLOCATOR.total_deallocations.load(Ordering::Relaxed));
    
    let slabs = ALLOCATOR.slabs.lock().unwrap();
    for &(size, ref slab) in slabs.iter() {
        println!("Slab size {}: {} used / {} total chunks", 
                 size, slab.used_chunks, slab.total_chunks);
    }
}