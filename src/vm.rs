use crate::instructions::Instruction;
use crate::orderbook::ShardedOrderbook;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct BulkBookVM {
    pub registers: [u64; 11],
    pub memory: Vec<u8>,
    pub program: Vec<Instruction>,
    pub pc: usize,
    pub orderbook: ShardedOrderbook,
    pub best_bid: AtomicU64,
    pub best_ask: AtomicU64,
}

impl BulkBookVM {
    pub fn new(program: Vec<Instruction>, shard_count: usize) -> Self {
        println!("Creating new BulkBookVM");
        println!("Program length: {}", program.len());
        println!("Shard count: {}", shard_count);
        
        let vm = BulkBookVM {
            registers: [0; 11],
            memory: vec![0; 1024],
            program,
            pc: 0,
            orderbook: ShardedOrderbook::new(shard_count),
            best_bid: AtomicU64::new(0),
            best_ask: AtomicU64::new(u64::MAX),
        };
        
        println!("BulkBookVM created successfully");
        vm
    }

    pub fn run(&mut self) {
        while self.pc < self.program.len() {
            let instruction = self.program[self.pc];
            self.execute(instruction);
            self.pc += 1;
        }
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Load(reg, value) => {
                self.registers[reg as usize] = value;
            },
            Instruction::Add(r1, r2, r3) => {
                self.registers[r3 as usize] = self.registers[r1 as usize] + self.registers[r2 as usize];
            },
            Instruction::Sub(r1, r2, r3) => {
                self.registers[r3 as usize] = self.registers[r1 as usize] - self.registers[r2 as usize];
            },
            Instruction::Mul(r1, r2, r3) => {
                self.registers[r3 as usize] = self.registers[r1 as usize] * self.registers[r2 as usize];
            },
            Instruction::Div(r1, r2, r3) => {
                self.registers[r3 as usize] = self.registers[r1 as usize] / self.registers[r2 as usize];
            },
            Instruction::PlaceOrderOptimized(price_reg, amount_reg, id_reg) => {
                let price = self.registers[price_reg as usize];
                let amount = self.registers[amount_reg as usize];
                let id = self.registers[id_reg as usize];
                self.orderbook.place_order(price, amount, id);
                self.update_best_bid_ask(price, amount);
            },
            Instruction::MatchOrdersInShard(shard_reg) => {
                let shard_id = self.registers[shard_reg as usize] as usize;
                self.match_orders_in_shard(shard_id);
            },
            Instruction::CrossShardMatch(shard1_reg, shard2_reg) => {
                let shard1 = self.registers[shard1_reg as usize] as usize;
                let shard2 = self.registers[shard2_reg as usize] as usize;
                self.cross_shard_match(shard1, shard2);
            },
            Instruction::UpdateBestBidAsk => {
                self.update_best_bid_ask_full();
            },
            Instruction::VectorizedPriceCheck(start_reg, end_reg, result_reg, shard_reg) => {
                let start = self.registers[start_reg as usize];
                let end = self.registers[end_reg as usize];
                let shard = self.registers[shard_reg as usize] as usize;
                let result = self.vectorized_price_check(start, end, shard);
                self.registers[result_reg as usize] = result;
            },
        }
    }

    fn update_best_bid_ask(&self, price: u64, amount: u64) {
        if amount > 0 {
            self.best_bid.fetch_max(price, Ordering::Relaxed);
        } else {
            self.best_ask.fetch_min(price, Ordering::Relaxed);
        }
    }

    fn update_best_bid_ask_full(&self) {
        for shard in &self.orderbook.shards {
            if let Some((&price, _)) = shard.iter().next_back() {
                self.best_bid.fetch_max(price, Ordering::Relaxed);
            }
            if let Some((&price, _)) = shard.iter().next() {
                self.best_ask.fetch_min(price, Ordering::Relaxed);
            }
        }
    }

    fn match_orders_in_shard(&mut self, shard_id: usize) {
        let shard = &mut self.orderbook.shards[shard_id];
        let mut matched = Vec::new();
        for (price, order) in shard.iter() {
            if order.amount.load(Ordering::Relaxed) > 0 {
                matched.push(*price);
            }
        }
        for price in matched {
            shard.remove(&price);
        }
    }

    fn cross_shard_match(&mut self, shard1: usize, shard2: usize) {
        let (left, right) = self.orderbook.shards.split_at_mut(shard1.max(shard2));
        let (shard1, shard2) = if shard1 < shard2 {
            (&mut left[shard1], &mut right[0])
        } else {
            (&mut right[0], &mut left[shard2])
        };
        
        let mut matched = Vec::new();
        for (price1, order1) in shard1.iter() {
            if let Some(order2) = shard2.get(price1) {
                if order1.amount.load(Ordering::Relaxed) > 0 && order2.amount.load(Ordering::Relaxed) > 0 {
                    matched.push(*price1);
                }
            }
        }
        for price in matched {
            shard1.remove(&price);
            shard2.remove(&price);
        }
    }

    fn vectorized_price_check(&self, start: u64, end: u64, shard: usize) -> u64 {
        self.orderbook.shards[shard]
            .range(start..=end)
            .map(|(_, order)| order.amount.load(Ordering::Relaxed))
            .sum()
    }
}