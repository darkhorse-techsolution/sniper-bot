use ethers::types::{
    Address,
    U256
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::utils::helpers::read_json;
use crate::utils::types::structs::token::Token;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnipeSettings {
    pub(crate) target_token: String,
    pub(crate) currency_token: String,
    pub(crate) slippage_tolerance: f32,
    pub(crate) gas_price: U256,
    pub(crate) amount_to_buy: U256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwapParams {
    pub(crate) token0: String,
    pub(crate) token1: String,
    pub(crate) version: u64,
    pub(crate) settings: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub(crate) target_token: Vec<Token>,
    pub(crate) slippage_tolerance: f64,
    pub(crate) gas_price: U256,
    pub(crate) token_in: Vec<Token>,
    pub(crate) buy_amount_tags: Vec<U256>,
}

impl Settings {
    pub fn new() -> Self {
        // Read and parse the settings JSON file
        let settings: Value = read_json("src/setting.json".parse().unwrap())
            .expect("Failed to read settings.json");

        // Parse target_tokens into a Vec<Address>
        let target_tokens = settings["target_tokens"]
            .as_array()
            .expect("target_tokens must be an array")
            .iter()
            .map(|token| {
                let address = token["address"]
                    .as_str()
                    .expect("Each target_token must have an address")
                    .parse::<Address>()
                    .expect("Invalid address in target_tokens");
                let symbol = token["symbol"]
                    .as_str()
                    .expect("Each token_in must have a symbol")
                    .to_string();
                let total_supply = token["total_supply"]
                    .as_u64() // Use `as_u64` for numeric values in JSON
                    .expect("Each target_token must have a total_supply")
                    .into(); // Convert to U256

                let decimals = token["decimals"]
                    .as_u64()
                    .expect("Each target_token must have a decimals")
                    .into();

                Token { address, symbol, decimals, total_supply }
            })
            .collect();

        let buy_amount_tags: Vec<U256> = settings["settings"]["buy_amount_tags"]
            .as_array()
            .expect("Buy amount tags must be an array")
            .iter()
            .map(|tag| U256::from(tag.as_u64().unwrap()))
            .collect();


        // Parse slippage_tolerance
        let slippage_tolerance = settings["settings"]["slippage"]
            .as_f64()
            .expect("slippage must be a floating point number");

        // Parse gas_price
        let gas_price = U256::from(
            settings["settings"]["gas_price"]
                .as_u64()
                .expect("gas_price must be a valid integer"),
        );

        let token_in = settings["settings"]["token_in"]
            .as_array()
            .expect("token_in must be an array")
            .iter()
            .map(|token| {
                let address = token["address"]
                    .as_str()
                    .expect("Each token_in must have an address")
                    .parse::<Address>()
                    .expect("Invalid address in token_in");
                let symbol = token["symbol"]
                    .as_str()
                    .expect("Each token_in must have a symbol")
                    .to_string();
                let total_supply = token["total_supply"]
                    .as_u64() // Use `as_u64` for numeric values in JSON
                    .expect("Each target_token must have a total_supply")
                    .into(); // Convert to U256

                let decimals = token["decimals"]
                    .as_u64()
                    .expect("Each target_token must have a decimals")
                    .into();

                Token { address, symbol, decimals, total_supply }
            })
            .collect();

        // Return the Settings instance
        Self {
            target_token: target_tokens,
            slippage_tolerance,
            gas_price,
            token_in,
            buy_amount_tags
        }
    }
}
