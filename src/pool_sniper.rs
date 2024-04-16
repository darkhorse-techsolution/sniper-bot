use std::str::FromStr;
use ethers::prelude::{abigen, LocalWallet};
use ethers::types::{Address, TransactionReceipt};
use ethers_core::types::U64;
use ethers_providers::Middleware;
use log::info;
use tokio::sync::broadcast::Receiver;
use crate::constants::Env;
use crate::utils::types::events::MemPoolEvent;
use crate::transaction_simulator::TransactionSimulator;
use crate::utils::helpers::{calculate_amount_in_v2_local, calculate_amount_out_v2_local, create_local_client, send_to_client, trade_swap_v1, trade_swap_v2};
use crate::utils::types::structs::setting::SnipeSettings;
use ethers::signers::Signer;

abigen!(
    PoolContract, // Name of the generated Rust struct
    r#"[{"inputs":[],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"owner","type":"address"},{"indexed":true,"internalType":"address","name":"spender","type":"address"},{"indexed":false,"internalType":"uint256","name":"value","type":"uint256"}],"name":"Approval","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"sender","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount0","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount1","type":"uint256"},{"indexed":true,"internalType":"address","name":"to","type":"address"}],"name":"Burn","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"sender","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount0","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount1","type":"uint256"}],"name":"Mint","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"sender","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount0In","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount1In","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount0Out","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount1Out","type":"uint256"},{"indexed":true,"internalType":"address","name":"to","type":"address"}],"name":"Swap","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"internalType":"uint112","name":"reserve0","type":"uint112"},{"indexed":false,"internalType":"uint112","name":"reserve1","type":"uint112"}],"name":"Sync","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"from","type":"address"},{"indexed":true,"internalType":"address","name":"to","type":"address"},{"indexed":false,"internalType":"uint256","name":"value","type":"uint256"}],"name":"Transfer","type":"event"},{"inputs":[],"name":"getReserves","outputs":[{"internalType":"uint112","name":"_reserve0","type":"uint112"},{"internalType":"uint112","name":"_reserve1","type":"uint112"},{"internalType":"uint32","name":"_blockTimestampLast","type":"uint32"}],"stateMutability":"view","type":"function"}]"#
);

abigen!(
    TokenContract,
    r#"[{"inputs":[{"internalType":"address","name":"","type":"address"},{"internalType":"address","name":"","type":"address"}],"name":"allowance","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"guy","type":"address"},{"internalType":"uint256","name":"wad","type":"uint256"}],"name":"approve","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"balanceOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"decimals","outputs":[{"internalType":"uint8","name":"","type":"uint8"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"deposit","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[],"name":"name","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"symbol","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"totalSupply","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"dst","type":"address"},{"internalType":"uint256","name":"wad","type":"uint256"}],"name":"transfer","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"src","type":"address"},{"internalType":"address","name":"dst","type":"address"},{"internalType":"uint256","name":"wad","type":"uint256"}],"name":"transferFrom","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"wad","type":"uint256"}],"name":"withdraw","outputs":[],"stateMutability":"nonpayable","type":"function"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":true,"name":"guy","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Approval","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"dst","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Deposit","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":true,"name":"dst","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Transfer","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Withdrawal","type":"event"},{"type":"receive"}]"#
);

pub struct PoolSniper {
    pub(crate) pool_address: Address,
    pub(crate) snipe_settings: SnipeSettings
}

impl PoolSniper {
    pub fn new(pool_address: Address, snipe_settings: SnipeSettings) -> Self {
        Self {
            pool_address,
            snipe_settings
        }
    }

    pub async fn snipe(&self, token0: Address, token1: Address, version: u64) -> TransactionReceipt {
        let client = create_local_client().await.clone();

        let env = Env::new();
        let sender = env
            .private_key
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(369_u64);
        let pool_contract = PoolContract::new(self.pool_address, client.clone());

        let nonce = client.get_transaction_count(sender.address(), None).await.unwrap();

        let snipe_setting = self.snipe_settings.clone();
        let currency_token = Address::from_str(&*snipe_setting.currency_token).unwrap();
        let target_token = Address::from_str(&*snipe_setting.target_token).unwrap();
        let input_token = if target_token == token0 { token1 } else { token0 };
        let currency_token_contract = TokenContract::new(currency_token, client.clone());

        let currency_decimal = currency_token_contract.decimals().call().await.unwrap_or(0_u8);

        println!("{} {} {} {} {}", currency_token, target_token, input_token, token0, token1);

        if input_token != currency_token {
            return TransactionReceipt::default();
        }
        let route = vec![currency_token, target_token];

        let (mut reserve_in, mut reserve_out, _) = pool_contract.get_reserves().call().await.unwrap();
        if currency_token > target_token {
            let temp = reserve_out.clone();
            reserve_out = reserve_in;
            reserve_in = temp;
        }

        let mut amount_to_buy = calculate_amount_in_v2_local(reserve_in, reserve_out, reserve_out / 50); // set 2% for now
        println!("snipe settings: {:?}", snipe_setting);
        amount_to_buy = snipe_setting.amount_to_buy.low_u128() * 10u128.pow(currency_decimal as u32);
        let amount_out = calculate_amount_out_v2_local(reserve_in, reserve_out, amount_to_buy);

        info!("amount out => {} {}", amount_to_buy, amount_out);

        let swap_message = serde_json::json!({
            "msg_type": "swapping",
            "content": {
                "pair_address": self.pool_address.clone(),
                "amount_in": amount_to_buy.to_string(),
                "amount_out": amount_out.to_string()
            }
        });
        send_to_client(swap_message.to_string(), true).await;
        let mut receipt = TransactionReceipt::default();

        if version == 1 {
            receipt = trade_swap_v1(route, amount_to_buy.into(), amount_out.into(), snipe_setting.gas_price, snipe_setting.slippage_tolerance, nonce).await;
        } else if version == 2 {
            receipt = trade_swap_v2(route, amount_to_buy.into(), amount_out.into(), snipe_setting.gas_price, snipe_setting.slippage_tolerance, nonce).await;
        }
        let swap_message = serde_json::json!({
            "msg_type": "swapped",
            "content": {
                "receipt": receipt,
                "pair_address": self.pool_address.clone()
            }
        });
        send_to_client(swap_message.to_string(), true).await;

        receipt
    }
}