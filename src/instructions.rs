#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Load(u8, u64),
    Add(u8, u8, u8),
    Sub(u8, u8, u8),
    Mul(u8, u8, u8),
    Div(u8, u8, u8),
    PlaceOrderOptimized(u8, u8, u8),
    MatchOrdersInShard(u8),
    CrossShardMatch(u8, u8),
    UpdateBestBidAsk,
    VectorizedPriceCheck(u8, u8, u8, u8),
}