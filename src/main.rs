use bulk_book_ebpf::vm::BulkBookVM;
use bulk_book_ebpf::instructions::Instruction;

fn main() {
    let program = vec![
        Instruction::Load(0, 100),  // Price
        Instruction::Load(1, 10),   // Amount
        Instruction::Load(2, 1),    // ID
        Instruction::PlaceOrderOptimized(0, 1, 2),
    ];

    let mut vm = BulkBookVM::new(program, 8);  // 8 shards
    
    println!("Initial state:");
    println!("Registers: {:?}", vm.registers);
    println!("Best bid: {}", vm.best_bid.load(std::sync::atomic::Ordering::Relaxed));
    println!("Best ask: {}", vm.best_ask.load(std::sync::atomic::Ordering::Relaxed));

    vm.run();

    println!("\nFinal state:");
    println!("Registers: {:?}", vm.registers);
    println!("Best bid: {}", vm.best_bid.load(std::sync::atomic::Ordering::Relaxed));
    println!("Best ask: {}", vm.best_ask.load(std::sync::atomic::Ordering::Relaxed));
    
    let shard = vm.orderbook.price_to_shard(100);
    println!("Orders in shard {}: {}", shard, vm.orderbook.shards[shard].len());
}