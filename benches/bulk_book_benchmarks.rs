use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bulk_book_ebpf::vm::BulkBookVM;
use bulk_book_ebpf::instructions::Instruction;

fn bench_order_placement(c: &mut Criterion) {
    c.bench_function("place 1000 orders", |b| {
        b.iter(|| {
            let mut vm = BulkBookVM::new(vec![], 8);
            for i in 0..1000 {
                vm.execute(Instruction::PlaceOrderOptimized(0, 1, 2));
                vm.registers[0] = black_box(100 + i);
                vm.registers[1] = black_box(10);
                vm.registers[2] = black_box(i);
            }
        })
    });
}

fn bench_vectorized_price_check(c: &mut Criterion) {
    c.bench_function("vectorized price check", |b| {
        let mut vm = BulkBookVM::new(vec![], 8);
        for i in 0..1000 {
            vm.orderbook.place_order(100 + i, 10, i);
        }
        b.iter(|| {
            vm.execute(Instruction::VectorizedPriceCheck(0, 1, 2, 3));
            vm.registers[0] = black_box(100);
            vm.registers[1] = black_box(1100);
            vm.registers[3] = black_box(0);
        })
    });
}

criterion_group!(benches, bench_order_placement, bench_vectorized_price_check);
criterion_main!(benches);