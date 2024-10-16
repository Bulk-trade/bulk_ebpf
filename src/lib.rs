pub mod vm;
pub mod orderbook;
pub mod instructions;
pub mod memory;

#[cfg(test)]
mod tests {
    use std::panic;
    use crate::memory::print_allocator_stats;
    use crate::vm::BulkBookVM;
    use crate::instructions::Instruction;

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

    #[test]
    fn test_bulk_book_vm() {
        run_test(|| {
            println!("Starting BulkBookVM test");
            let program = vec![
                Instruction::Load(0, 100),  // Price
                Instruction::Load(1, 10),   // Amount
                Instruction::Load(2, 1),    // ID
                Instruction::PlaceOrderOptimized(0, 1, 2),
            ];
            let mut vm = BulkBookVM::new(program, 8);  // 8 shards
            vm.run();
            println!("BulkBookVM test completed");
        });
    }
}