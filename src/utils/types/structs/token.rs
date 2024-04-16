use ethers_core::types::{Address, Uint8, U256};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    pub address: Address,
    pub total_supply: U256,
    pub decimals: U256,
    pub symbol: String,
}

impl Token {
    pub fn new(address: Address, total_supply: U256, decimals: U256, symbol: String) -> Self {
        Self {
            address,
            total_supply,
            decimals,
            symbol
        }
    }
}