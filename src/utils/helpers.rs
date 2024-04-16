use std::cmp::max;
use ethers::types::{
    TransactionReceipt, U256
};
use fern::Dispatch;
use chrono::Local;
use log::{info, warn, error, debug};
use serde_json::{to_string_pretty, Value};
use std::fs::File;
use std::future::IntoFuture;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use cached::proc_macro::cached;
use ethers::contract::Multicall;
use ethers::prelude::{abigen, TxHash};
use ethers::signers::{LocalWallet, Signer};
use ethers_core::abi::ParamType;
use ethers_core::types::{Address, Block, Bytes, Eip1559TransactionRequest, NameOrAddress, U64};
use ethers_core::types::transaction::eip2718::TypedTransaction;
use ethers_core::types::transaction::eip2930::AccessList;
use ethers_providers::{Http, Middleware, Provider, Ws};
use tokio::fs;
use crate::bot::ClientMessage;
use crate::constants::{Env, GWEI, NATIVE_TOKEN, PULSEX_SWAP_ROUTER, PULSEX_SWAP_ROUTER_V2, WEBSOCKET_ADDR, WEB_HOOK_URL};
use crate::utils::types::structs::setting::Settings;
use reqwest::Client;
use serde_json::json;
use std::future::Future;
use std::ops::Add;
use std::pin::Pin;
use crate::pool_sniper::TokenContract;

abigen!(PulseXRouter, r#"[{"inputs":[{"internalType":"address","name":"_factoryV1","type":"address"},{"internalType":"address","name":"factoryV2","type":"address"},{"internalType":"address","name":"_stableInfo","type":"address"},{"internalType":"address","name":"_WETH9","type":"address"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"factory","type":"address"},{"indexed":true,"internalType":"address","name":"info","type":"address"}],"name":"SetStableSwap","type":"event"},{"inputs":[],"name":"WETH9","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address[]","name":"swapContracts","type":"address[]"},{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"}],"name":"exactInputStableSwap","outputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address[]","name":"swapContracts","type":"address[]"},{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"uint256","name":"amountInMax","type":"uint256"},{"internalType":"address","name":"to","type":"address"}],"name":"exactOutputStableSwap","outputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"}],"stateMutability":"payable","type":"function"},{"inputs":[],"name":"factoryV1","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"factoryV2","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"bytes32","name":"previousBlockhash","type":"bytes32"},{"internalType":"bytes[]","name":"data","type":"bytes[]"}],"name":"multicall","outputs":[{"internalType":"bytes[]","name":"","type":"bytes[]"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"deadline","type":"uint256"},{"internalType":"bytes[]","name":"data","type":"bytes[]"}],"name":"multicall","outputs":[{"internalType":"bytes[]","name":"","type":"bytes[]"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"bytes[]","name":"data","type":"bytes[]"}],"name":"multicall","outputs":[{"internalType":"bytes[]","name":"results","type":"bytes[]"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"value","type":"uint256"}],"name":"pull","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[],"name":"refundETH","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"value","type":"uint256"},{"internalType":"uint256","name":"deadline","type":"uint256"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"selfPermit","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"nonce","type":"uint256"},{"internalType":"uint256","name":"expiry","type":"uint256"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"selfPermitAllowed","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"nonce","type":"uint256"},{"internalType":"uint256","name":"expiry","type":"uint256"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"selfPermitAllowedIfNecessary","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"value","type":"uint256"},{"internalType":"uint256","name":"deadline","type":"uint256"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"selfPermitIfNecessary","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[],"name":"stableSwapInfo","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"}],"name":"swapExactTokensForTokensV1","outputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"}],"name":"swapExactTokensForTokensV2","outputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"uint256","name":"amountInMax","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"}],"name":"swapTokensForExactTokensV1","outputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"uint256","name":"amountInMax","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"}],"name":"swapTokensForExactTokensV2","outputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"amountMinimum","type":"uint256"},{"internalType":"address","name":"recipient","type":"address"}],"name":"sweepToken","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"amountMinimum","type":"uint256"}],"name":"sweepToken","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"amountMinimum","type":"uint256"},{"internalType":"uint256","name":"feeBips","type":"uint256"},{"internalType":"address","name":"feeRecipient","type":"address"}],"name":"sweepTokenWithFee","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"amountMinimum","type":"uint256"},{"internalType":"address","name":"recipient","type":"address"},{"internalType":"uint256","name":"feeBips","type":"uint256"},{"internalType":"address","name":"feeRecipient","type":"address"}],"name":"sweepTokenWithFee","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountMinimum","type":"uint256"},{"internalType":"address","name":"recipient","type":"address"}],"name":"unwrapWETH9","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountMinimum","type":"uint256"}],"name":"unwrapWETH9","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountMinimum","type":"uint256"},{"internalType":"address","name":"recipient","type":"address"},{"internalType":"uint256","name":"feeBips","type":"uint256"},{"internalType":"address","name":"feeRecipient","type":"address"}],"name":"unwrapWETH9WithFee","outputs":[],"stateMutability":"payable","type":"function"},{"stateMutability":"payable","type":"receive"}]"#);
abigen!(PulseXRouterV2, r#"[{
    "inputs": [
        {
            "internalType": "uint256",
            "name": "amountOutMin",
            "type": "uint256"
        },
        {
            "internalType": "address[]",
            "name": "path",
            "type": "address[]"
        },
        {
            "internalType": "address",
            "name": "to",
            "type": "address"
        },
        {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }
    ],
    "name": "swapExactETHForTokens",
    "outputs": [
        {
            "internalType": "uint256[]",
            "name": "amounts",
            "type": "uint256[]"
        }
    ],
    "stateMutability": "payable",
    "type": "function"
}]"#);

abigen!(
    TokenContract,
    r#"[{"inputs":[{"internalType":"address","name":"","type":"address"},{"internalType":"address","name":"","type":"address"}],"name":"allowance","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"guy","type":"address"},{"internalType":"uint256","name":"wad","type":"uint256"}],"name":"approve","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"balanceOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"decimals","outputs":[{"internalType":"uint8","name":"","type":"uint8"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"deposit","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[],"name":"name","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"symbol","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"totalSupply","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"dst","type":"address"},{"internalType":"uint256","name":"wad","type":"uint256"}],"name":"transfer","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"src","type":"address"},{"internalType":"address","name":"dst","type":"address"},{"internalType":"uint256","name":"wad","type":"uint256"}],"name":"transferFrom","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"wad","type":"uint256"}],"name":"withdraw","outputs":[],"stateMutability":"nonpayable","type":"function"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":true,"name":"guy","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Approval","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"dst","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Deposit","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":true,"name":"dst","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Transfer","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Withdrawal","type":"event"},{"type":"receive"}]"#
);

pub fn calculate_amount_out_v2_local(reserve_in: u128, reserve_out: u128, amount_in: u128) -> u128 {
    println!("reserveIn, reserveOut => {} {}", reserve_in, reserve_out);
    let mut _reserve_in = U256::from(reserve_in);
    let _reserve_out = U256::from(reserve_out);
    let mut _amount_in = U256::from(amount_in);
    let amount_in_with_fee = _amount_in * 9971; // Fee is 0.29% (10000 - 29 = 9971)
    let numerator = amount_in_with_fee * _reserve_out;
    let denominator = _reserve_in * 10000 + amount_in_with_fee;
    let res: U256 = numerator / denominator;

    info!("reserves => {} {} {} {}", reserve_in, reserve_out, amount_in, res.low_u128());
    res.low_u128()
}

pub(crate) fn calculate_amount_in_v2_local(reserve_in: u128, reserve_out: u128, amount_out: u128) -> u128 {
    // Define the fee numerator and denominator
    let fee_numerator = U256::from(9971); // 0.9979
    let fee_denominator = U256::from(10000); // 1.0 (scaled by 10000)
    let mut _reserve_in = U256::from(reserve_in);
    let _reserve_out = U256::from(reserve_out);
    let mut _amount_out = U256::from(amount_out);

    // Ensure amountOut is valid
    if amount_out >= reserve_out {
        panic!("Amount out exceeds or equals reserve out, invalid operation.");
    }

    // Calculate the numerator and denominator
    let numerator = _reserve_in * _amount_out * fee_denominator;
    let denominator = (_reserve_out - _amount_out) * fee_numerator;

    // Perform the division and return the result
    (numerator / denominator).low_u128() + 1// Add 1 to round up if needed
}

pub fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
    Dispatch::new()
        // Set the default logging level
        .level(log::LevelFilter::Info)
        // Configure logging to file
        .chain(fern::log_file("web_server.log")?)
        // Configure logging to console
        .chain(std::io::stdout())
        // Format log messages with time and log level
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .apply()?;
    tokio::spawn(async move {
        let send_message = setup_websocket_client(*WEB_HOOK_URL);
        send_message("start").await;
    });
    Ok(())
}

pub async fn trade_swap_v1(path: Vec<Address>, amount_in: U256, amount_out: U256, gas_price: U256, slippage: f32, nonce: U256) -> TransactionReceipt {
    let env = Env::new();
    let client = create_local_client().await.clone();

    let pulse_x_router = PulseXRouter::new(*PULSEX_SWAP_ROUTER, client.clone());
    let pulse_x_router_v2 = PulseXRouterV2::new(*PULSEX_SWAP_ROUTER_V2, client.clone());
    let mut call_data = Bytes::new();
    let mut value = U256::zero();
    let mut to = *PULSEX_SWAP_ROUTER;
    let _amount_out = amount_out * U256::from(1000 - (slippage * 10_f32) as u32) / U256::from(1000);

    if path[0] == *NATIVE_TOKEN {
        call_data = pulse_x_router_v2.swap_exact_eth_for_tokens(_amount_out, path.clone(), Address::from_str(&*env.public_key).unwrap(), U256::max_value())
            .calldata()
            .unwrap();
        value = amount_in;
        to = *PULSEX_SWAP_ROUTER_V2;
        let balance = client.get_balance(Address::from_str(&*env.public_key).unwrap(), None).await.unwrap();

        if balance < amount_in {
            let mut receipt = TransactionReceipt::default();
            receipt.status = Some(U64::from(0));
            return receipt;
        }
    } else {
        call_data = pulse_x_router.swap_exact_tokens_for_tokens_v1(amount_in, _amount_out, path.clone(), Address::from_str(&*env.public_key).unwrap())
            .calldata()
            .unwrap();
    }

    let tx = build_eip1559_tx(to, call_data.clone(), value, gas_price, nonce);
    info!("Swapping ... {} {}", amount_in, _amount_out);
    println!("unsigned tx => {:?}", tx);
    let signed_tx = sign_tx(tx).await.unwrap();
    println!("signed tx => {}", signed_tx);
    let res = client.send_raw_transaction(signed_tx).await.unwrap();

    let receipt = res.into_future().await.unwrap().unwrap();
    println!("receipt => {:?}", receipt);

    // println!("{:?}", res.tx_hash());
    receipt
}

pub async fn approve(token_address: Address, target_address: Address, amount: U256) {
    let env = Env::new();

    let client = create_local_client().await.clone();

    let pub_key = Address::from_str(&*env.public_key.clone()).unwrap();
    let currency_token_contract = TokenContract::new(token_address, client.clone());
    let token_balance = currency_token_contract.balance_of(pub_key).await.unwrap_or(U256::zero());

    let call_data = currency_token_contract.approve(target_address, amount)
        .calldata()
        .unwrap();

    let sender = env
        .private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(369_u64);
    let nonce = client.get_transaction_count(sender.address(), None).await.unwrap();

    let tx = build_eip1559_tx(token_address, call_data.clone(), U256::zero(), U256::from(5000000), nonce);
    let signed_tx = sign_tx(tx).await.unwrap();
    println!("signed tx => {} {}", target_address, token_balance);
    let res = client.send_raw_transaction(signed_tx).await.unwrap();

    let receipt = res.into_future().await.unwrap().unwrap();
    println!("receipt => {:?}", receipt);
}

pub async fn get_allowance(token_address: Address) -> U256 {
    let env = Env::new();
    let client = create_local_client().await.clone();

    let currency_token_contract = TokenContract::new(token_address, client.clone());
    let sender = env
        .private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(369_u64);

    let allowance = currency_token_contract.allowance(sender.address(), *PULSEX_SWAP_ROUTER).call().await.unwrap();
    allowance
}

pub async fn trade_swap_v2(path: Vec<Address>, amount_in: U256, amount_out: U256, gas_price: U256, slippage: f32, nonce: U256) -> TransactionReceipt {
    let env = Env::new();
    let client = create_local_client().await.clone();

    let pulse_x_router = PulseXRouter::new(*PULSEX_SWAP_ROUTER, client.clone());
    let pulse_x_router_v2 = PulseXRouterV2::new(*PULSEX_SWAP_ROUTER_V2, client.clone());
    let mut call_data = Bytes::new();
    let mut value = U256::zero();
    let mut to = *PULSEX_SWAP_ROUTER;
    let _amount_out = amount_out * U256::from(1000 - (slippage * 10_f32) as u32) / U256::from(1000);

    if path[0] == *NATIVE_TOKEN {
        call_data = pulse_x_router_v2.swap_exact_eth_for_tokens(_amount_out, path.clone(), Address::from_str(&*env.public_key).unwrap(), U256::max_value())
            .calldata()
            .unwrap();
        value = amount_in;
        to = *PULSEX_SWAP_ROUTER_V2;
        let balance = client.get_balance(Address::from_str(&*env.public_key).unwrap(), None).await.unwrap();
        if balance < amount_in {
            let mut receipt = TransactionReceipt::default();
            receipt.status = Some(U64::from(0));
            return receipt;
        }
    } else {
        call_data = pulse_x_router.swap_exact_tokens_for_tokens_v2(amount_in, _amount_out, path.clone(), Address::from_str(&*env.public_key).unwrap())
            .calldata()
            .unwrap();
    }

    let tx = build_eip1559_tx(to, call_data.clone(), value, gas_price, nonce);
    info!("Swapping ... {} {}", amount_in, _amount_out);
    println!("unsigned tx => {:?}", tx);
    let signed_tx = sign_tx(tx).await.unwrap();
    println!("signed tx => {}", signed_tx);

    let res = client.send_raw_transaction(signed_tx).await.unwrap();

    let receipt = res.into_future().await.unwrap().unwrap();

    println!("receipt => {:?}", receipt);

    // println!("{:?}", res.tx_hash());
    // receipt.status.unwrap_or(U64::zero())
    receipt
}

pub fn build_eip1559_tx(to: Address, data: Bytes, value: U256, gas_price: U256, nonce: U256) -> Eip1559TransactionRequest {
    let env = Env::new();
    let adjusted_gas_price = max(gas_price.clone(), U256::from(1000000));

    let tx = Eip1559TransactionRequest {
        to: Some(NameOrAddress::from(to)),
        from: Some(Address::from_str(&*env.public_key.clone()).unwrap()),
        data: Some(Bytes::from(data)),
        value: Some(value),
        chain_id: Some(U64::from(369_u64)),
        gas: Some(U256::from(6000000)),
        nonce: Some(U256::from(nonce)),
        max_fee_per_gas: Some(adjusted_gas_price * *GWEI),
        max_priority_fee_per_gas: Some(adjusted_gas_price * *GWEI),
        access_list: AccessList::default()
    };

    tx
}

pub async fn sign_tx(tx: Eip1559TransactionRequest) -> anyhow::Result<Bytes> {
    let env = Env::new();
    let typed = TypedTransaction::Eip1559(tx);
    let sender = env
        .private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(369_u64);

    let signature = sender.sign_transaction(&typed).await?;
    let signed = typed.rlp_signed(&signature);
    Ok(signed)
}

pub fn read_json(path: String) -> Result<Value, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;
    let json_data: Value = serde_json::from_str(&contents)?;

    Ok(json_data)
}

pub async fn write_json_to_file(file_path: &str, data: Value) -> std::io::Result<()> {
    // Create the directory if it doesn't exist
    if let Some(parent) = Path::new(file_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    // Serialize the data to JSON
    let json_data = to_string_pretty(&data).unwrap();

    // Write the JSON to the file
    let mut file = File::create(file_path)?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}

pub fn calculate_next_block_base_fee(block: Block<TxHash>) -> U256 {
    // Get the block base fee per gas
    let current_base_fee_per_gas = block.base_fee_per_gas.unwrap_or_default();

    // Get the mount of gas used in the block
    let current_gas_used = block.gas_used;

    let current_gas_target = block.gas_limit / 2;

    if current_gas_used == current_gas_target {
        current_base_fee_per_gas
    } else if current_gas_used > current_gas_target {
        let gas_used_delta = current_gas_used - current_gas_target;
        let base_fee_per_gas_delta =
            current_base_fee_per_gas * gas_used_delta / current_gas_target / 8;

        return current_base_fee_per_gas + base_fee_per_gas_delta;
    } else {
        let gas_used_delta = current_gas_target - current_gas_used;
        let base_fee_per_gas_delta =
            current_base_fee_per_gas * gas_used_delta / current_gas_target / 8;

        return current_base_fee_per_gas - base_fee_per_gas_delta;
    }
}

#[cached]
pub async fn create_local_client() -> Arc<Provider<Ws>> {
    let env = Env::new();
    println!("websocket => {}", env.wss_url);
    let wss_url = "wss://rpc.pulsechain.com";
    let client = Provider::<Ws>::connect(wss_url).await.unwrap();
    Arc::new(client)
}


#[cached]
pub async fn get_multicall() -> Multicall<Provider<Ws>> {
    let env = Env::new();
    let client = Provider::<Ws>::connect(env.wss_url).await.unwrap();

    Multicall::new(client.clone(), None).await.unwrap()
}

pub fn decode_pair_create_method_input(input: Bytes) -> (Address, Address) {
    let input_data = input.clone();
    let params_data = &input_data[4..];

    let param_types = vec![
        ParamType::Address,
        ParamType::Address
    ];

    let decoded_params = ethers::abi::decode(&param_types, params_data)
        .expect("Decoding failed");

    let token0 = decoded_params[0].clone().into_address().unwrap_or(Address::zero());
    let token1 = decoded_params[1].clone().into_address().unwrap_or(Address::zero());
    (token0, token1)
}

pub async fn send_to_client(message: String, will_print: bool) {
    if (will_print) {
        println!("Sending to client: {}", message);
    }

    let addr = WEBSOCKET_ADDR.lock().await;
    if let Some(ref websocket_addr) = *addr {
        websocket_addr.do_send(ClientMessage(message));
    }
}

pub async fn send_message_to_client() {
    // let env = Env::new();
    // let send_message = setup_websocket_client(*WEB_HOOK_URL);
    // send_message(format!("{:?}", env).as_str()).await;
}

pub fn setup_websocket_client(
    webhook_url: &str,
) -> impl Fn(&str) -> Pin<Box<dyn Future<Output = ()> + Send>> + '_ {
    let webhook_url = webhook_url.to_string();

    move |message: &str| {
        let webhook_url = webhook_url.clone();
        let message = message.to_string();

        Box::pin(async move {
            let client = Client::new();
            let payload = json!({ "content": message });

            match client.post(&webhook_url).json(&payload).send().await {
                Ok(response) => {
                    if response.status() != reqwest::StatusCode::NO_CONTENT {
                        eprintln!(
                            "Failed to send message: {:?}",
                            response.text().await.unwrap_or_default()
                        );
                    } else {
                    }
                }
                Err(e) => eprintln!("Request failed: {}", e),
            }
        })
    }
}
