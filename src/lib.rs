pub mod vm;
pub mod orderbook;
pub mod instructions;
pub mod memory;

#[cfg(test)]
mod tests {
    use crate::memory::print_allocator_stats;

    #[test]
    fn test_vm_creation() {
        use crate::vm::BulkBookVM;
        use crate::instructions::Instruction;
        println!("Testing VM creation");
        let program = vec![Instruction::Load(0, 100)];
        let _vm = BulkBookVM::new(program, 8);
        println!("VM created successfully");
        print_allocator_stats();
    }

    #[test]
    fn test_place_order() {
        use crate::vm::BulkBookVM;
        use crate::instructions::Instruction;
        use std::sync::atomic::Ordering;

        println!("Testing place order");
        let program = vec![
            Instruction::Load(0, 100),  // Price
            Instruction::Load(1, 10),   // Amount
            Instruction::Load(2, 1),    // ID
            Instruction::PlaceOrderOptimized(0, 1, 2),
        ];
        let mut vm = BulkBookVM::new(program, 8);
        vm.run();
        assert_eq!(vm.best_bid.load(Ordering::Relaxed), 100);
        assert_eq!(vm.orderbook.shards[vm.orderbook.price_to_shard(100)].len(), 1);
        println!("Place order test completed");
        print_allocator_stats();
    }

    
}