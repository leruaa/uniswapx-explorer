use stapifaction::Persistable;
use uniswapx::types::{OrderStatus, OrderType};

use crate::{OrderAsset, OrderDetails};

#[derive(Persistable, Debug)]
#[persistable(path = "orders")]
#[serde(rename_all = "camelCase")]
pub struct Order {
    #[persistable(id)]
    pub hash: String,
    pub chain_id: u64,
    pub created_at: u64,
    #[serde(rename = "type")]
    pub ty: OrderType,
    pub status: OrderStatus,
    pub input: OrderAsset,
    pub output: OrderAsset,
    pub fee: Option<OrderAsset>,
    pub recipient: String,
    pub signature: String,
    pub tx: Option<String>,
    #[persistable(expand)]
    pub details: Option<OrderDetails>,
}
