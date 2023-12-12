use erc20::Token;
use num_traits::cast::ToPrimitive;
use primitive_types::U256;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderAsset {
    pub start_amount: f64,
    pub end_amount: f64,
    pub symbol: String,
    pub price: f64,
}

impl OrderAsset {
    pub fn new(token: &Arc<Token>, start_amount: U256, end_amount: U256, price: f64) -> Self {
        Self {
            start_amount: token.get_balance(start_amount).to_f64().unwrap(),
            end_amount: token.get_balance(end_amount).to_f64().unwrap(),
            symbol: token.symbol.clone(),
            price,
        }
    }
}
