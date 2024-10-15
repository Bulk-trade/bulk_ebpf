use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::cmp::Ordering as CmpOrdering;

#[derive(Debug)]
#[repr(C, align(64))]
pub struct CacheAlignedOrder {
    pub price: AtomicU64,
    pub amount: AtomicU64,
    pub id: u64,
    padding: [u8; 40],
}

impl PartialEq for CacheAlignedOrder {
    fn eq(&self, other: &Self) -> bool {
        self.price.load(Ordering::Relaxed) == other.price.load(Ordering::Relaxed)
            && self.amount.load(Ordering::Relaxed) == other.amount.load(Ordering::Relaxed)
            && self.id == other.id
    }
}

impl Eq for CacheAlignedOrder {}

impl PartialOrd for CacheAlignedOrder {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for CacheAlignedOrder {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        self.price
            .load(Ordering::Relaxed)
            .cmp(&other.price.load(Ordering::Relaxed))
            .then_with(|| self.id.cmp(&other.id))
    }
}

pub struct ShardedOrderbook {
    pub shards: Vec<BTreeMap<u64, CacheAlignedOrder>>,
    pub shard_count: usize,
}

impl ShardedOrderbook {
    pub fn new(shard_count: usize) -> Self {
        ShardedOrderbook {
            shards: (0..shard_count).map(|_| BTreeMap::new()).collect(),
            shard_count,
        }
    }

    pub fn place_order(&mut self, price: u64, amount: u64, id: u64) {
        let shard_index = self.price_to_shard(price);
        self.shards[shard_index].insert(price, CacheAlignedOrder {
            price: AtomicU64::new(price),
            amount: AtomicU64::new(amount),
            id,
            padding: [0; 40],
        });
    }

    pub fn price_to_shard(&self, price: u64) -> usize {
        (price as usize) % self.shard_count
    }
}