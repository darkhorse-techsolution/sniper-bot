use ethers::{
    prelude::Lazy,
    types::{Address, U256, U64},
};
use std::str::FromStr;
use std::sync::Arc;
use actix::Addr;
use lazy_static::lazy_static;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Receiver;
use crate::bot::{Bot, FactoryV2Contract, MyWebSocket};
use crate::utils::helpers::create_local_client;
use crate::utils::types::structs::pool::Pool;
use crate::utils::types::structs::setting::{SnipeSettings, SwapParams};

pub static GWEI: Lazy<U256> = Lazy::new(|| U256::from(10).pow(U256::from(9)));
pub static PULSEX_FACTORY_ADDRESS_V1: Lazy<Address> =
    Lazy::new(|| Address::from_str("0x1715a3E4A142d8b698131108995174F37aEBA10D").unwrap());
pub static PULSEX_FACTORY_ADDRESS_V2: Lazy<Address> =
    Lazy::new(|| Address::from_str("0x29eA7545DEf87022BAdc76323F373EA1e707C523").unwrap());
pub static PULSEX_SWAP_ROUTER: Lazy<Address> =
    Lazy::new(|| Address::from_str("0xDA9aBA4eACF54E0273f56dfFee6B8F1e20B23Bba").unwrap());

pub static PULSEX_SWAP_ROUTER_V2: Lazy<Address> =
    Lazy::new(|| Address::from_str("0x165C3410fC91EF562C50559f7d2289fEbed552d9").unwrap());

pub static NATIVE_TOKEN: Lazy<Address> =
    Lazy::new(|| Address::from_str("0xa1077a294dde1b09bb078844df40758a5d0f9a27").unwrap());

pub static PAIR_CREATED_ABI: Lazy<&str> = Lazy::new(|| r#"[{
    "anonymous": false,
    "inputs": [
        {"indexed": true, "internalType": "address", "name": "token0", "type": "address"},
        {"indexed": true, "internalType": "address", "name": "token1", "type": "address"},
        {"indexed": false, "internalType": "address", "name": "pair", "type": "address"},
        {"indexed": false, "internalType": "uint256", "name": "pair_count", "type": "uint256"}
    ],
    "name": "PairCreated",
    "type": "event"
}]"#);

pub static WEB_HOOK_URL: Lazy<&str> = Lazy::new(|| "https://discord.com/api/webhooks/1283782021013045381/aE8KybyX2r8wzln7BLacWXl9sF5PUPiNa_Ydt9LvSJ-WhvkE02E2kCWmQUf-NeeHKZms");

pub fn get_env(key: &str) -> String {
    std::env::var(key).unwrap()
}
#[derive(Debug, Clone)]
pub struct Env {
    pub private_key: String,
    pub public_key: String,
    pub rpc_url: String,
    pub port: String,
    // pub bot_address: String,
    pub chain_id: U64,
    pub wss_url: String,
}

impl Env {
    pub fn new() -> Self {
        let private_key = get_env("PRIVATE_KEY");
        let public_key = get_env("PUBLIC_KEY");

        // private_key = decode_string(private_key);
        Env {
            rpc_url: get_env("RPC_URL"),
            chain_id: U64::from_str(&get_env("CHAIN_ID")).unwrap(),
            private_key,
            public_key,
            port: get_env("PORT"),
            // bot_address: get_env("BOT_ADDRESS"),
            wss_url: get_env("WSS_URL"),
        }
    }
}

// #[derive(Debug, Clone)]
pub struct SwapHandler {
    sender: mpsc::Sender<SwapParams>,
}

impl SwapHandler {
    pub fn new() -> (Self, Receiver<SwapParams>) {
        let (sender, receiver) = mpsc::channel(100); // Buffer size of 100
        (Self { sender }, receiver)
    }

    pub async fn handle_swap(&self, swap_params: SwapParams) {
        if let Err(e) = self.sender.send(swap_params).await {
            eprintln!("Failed to queue swap request: {}", e);
        }
    }
}

lazy_static! {
    pub static ref WEBSOCKET_ADDR: Arc<Mutex<Option<Addr<MyWebSocket>>>> =
        Arc::new(Mutex::new(None));
}

lazy_static! {
    // Global constants for rate limiting and message queue
    pub static ref SWAP_MESSAGE_QUEUE: Arc<Mutex<mpsc::Sender<SwapParams>>> = {
        let (tx, rx) = mpsc::channel::<SwapParams>(100);
        let queue = Arc::new(Mutex::new(tx));
        tokio::spawn(process_swap_queue(rx));
        queue
    };
}

pub async fn process_swap_queue(mut rx: mpsc::Receiver<SwapParams>) {
    while let Some(data) = rx.recv().await {
        let mut bot = Bot::new().await;

        let client = create_local_client().await.clone();

        let token0 = Address::from_str(&*data.token0).unwrap();
        let token1 = Address::from_str(&*data.token1).unwrap();

        let factory_v2 = FactoryV2Contract::new(*PULSEX_FACTORY_ADDRESS_V2, client);
        let pair_address = factory_v2.get_pair(token0, token1).call().await.unwrap();

        let version = data.version;
        let settings = serde_json::from_str::<SnipeSettings>(&data.settings).unwrap();

        let pool = Pool::new(pair_address, token0, token1, U256::zero(), version);

        bot.direct_swap(pool, settings).await.unwrap();
    }
}