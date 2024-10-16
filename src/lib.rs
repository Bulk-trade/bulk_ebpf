pub mod vm;
pub mod orderbook;
pub mod instructions;
pub mod memory;

#[cfg(test)]
mod tests {
    use std::panic;
    use crate::memory::print_allocator_stats;

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce() -> () + panic::UnwindSafe,
    {
        let result = panic::catch_unwind(|| {
            test()
        });

        match result {
            Ok(_) => println!("Test passed successfully"),
            Err(e) => println!("Test panicked: {:?}", e),
        }

        print_allocator_stats();
    }

    #[test]
    fn test_basic_allocation() {
        run_test(|| {
            println!("Starting basic allocation test");
            let vec: Vec<u8> = vec![0; 1000];
            println!("Allocated vector of size {}", vec.len());
        });
    }

    #[test]
    fn test_multiple_allocations() {
        run_test(|| {
            println!("Starting multiple allocations test");
            for i in 0..10 {
                let vec: Vec<u8> = vec![0; 100 * (i + 1)];
                println!("Allocated vector of size {}", vec.len());
            }
        });
    }

    // Uncomment and add more tests as needed
    // pub mod unit_tests;
    // pub mod integration_tests;
}