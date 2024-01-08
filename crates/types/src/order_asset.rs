use alloy_primitives::U256;
use erc20::Token;
use num_traits::cast::ToPrimitive;
use serde::Serialize;
use std::sync::Arc;
use tsify::Tsify;

#[derive(Debug, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct OrderAsset {
    pub start_amount: f64,
    pub end_amount: f64,
    pub settled_amount: Option<f64>,
    pub symbol: String,
    pub price: f64,
}

impl OrderAsset {
    pub fn new(
        token: &Arc<Token>,
        start: U256,
        end: U256,
        settled: Option<U256>,
        price: f64,
    ) -> Self {
        Self {
            start_amount: token.get_balance(start).to_f64().unwrap(),
            end_amount: token.get_balance(end).to_f64().unwrap(),
            settled_amount: settled.map(|a| token.get_balance(a).to_f64().unwrap()),
            symbol: token.symbol.clone(),
            price,
        }
    }
}
