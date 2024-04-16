use chrono::format::{DelayedFormat, StrftimeItems};
use ethers::prelude::*;
use serde_json::Value;
use chrono::prelude::*;

// Holds Pool Information
#[derive(Clone, Debug, PartialEq)]
pub struct Pool {
    pub address: Address,
    pub token_0: Address,
    pub token_1: Address,
    pub weth_liquidity: U256,
    pub version: u64,
    pub launch_time: String,
}

impl Pool {
    pub fn new(address: Address, token_a: Address, token_b: Address, weth_liquidity: U256, version: u64) -> Pool {
        let token_0 = token_a;
        let token_1 = token_b;
        let launch_time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

        Pool {
            address,
            token_0,
            token_1,
            weth_liquidity,
            version,
            launch_time
        }
    }

    pub fn to_value(&self) -> Value {
        serde_json::json!({
            "address": format!("{:?}", self.address), // Convert Address to a string
            "token_0": format!("{:?}", self.token_0), // Convert Address to a string
            "token_1": format!("{:?}", self.token_1), // Convert Address to a string
            "weth_liquidity": self.weth_liquidity.to_string(), // Convert U256 to a string
            "version": self.version,
            "launch_time": format!("{:?}", self.launch_time),
        })
    }
}