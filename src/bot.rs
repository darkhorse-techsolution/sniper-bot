use crate::constants::{
    Env, PAIR_CREATED_ABI, PULSEX_FACTORY_ADDRESS_V1, PULSEX_FACTORY_ADDRESS_V2,
    SWAP_MESSAGE_QUEUE, WEBSOCKET_ADDR,
};
use crate::pool_sniper::PoolSniper;
use crate::token_sniper::TokenSniper;
use actix::{Actor, ActorContext, Handler, Message, StreamHandler};
use ethers::prelude::{TxHash, H256, U256, U64};
use ethers::providers::Middleware;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, TransactionReceipt};
use ethers_core::types::{Block, Filter, Topic, Transaction};
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use ethers_providers::StreamExt;
use log::info;
use tokio::sync::{broadcast, Mutex};

use ethers::abi::Abi;

use crate::utils::helpers::{
    calculate_next_block_base_fee, create_local_client, send_message_to_client, send_to_client,
    trade_swap_v2,
};
use crate::utils::types::events::{NewPoolEvent, NewTokenEvent};
use crate::utils::types::structs::pool::Pool;
use crate::utils::types::structs::setting::{Settings, SnipeSettings, SwapParams};
use crate::utils::types::structs::token::Token;
use actix_web_actors::ws;
use ethers::contract::abigen;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize, Debug)]
struct MyMessage {
    msg_type: String,
    content: String,
}

pub struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    // When the WebSocket connection starts, we can send an initial message.
    fn started(&mut self, ctx: &mut Self::Context) {
        let env = Env::new();

        println!("Client connected!");
        // Send a welcome message to the client
        self.send_event(ctx, "Welcome to the server!");

        tokio::spawn(async move {
            let message = serde_json::json!({
                "msg_type": "get wallet address",
                "content": {
                    "wallet_address": env.public_key
                }
            });
            send_to_client(message.to_string(), true).await;
        });
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        println!("Client disconnected!");
        actix::Running::Stop
    }
}

impl MyWebSocket {
    // Function to send a message to the WebSocket client
    pub fn send_event(&self, ctx: &mut ws::WebsocketContext<Self>, event: &str) {
        let message = MyMessage {
            msg_type: "server_event".to_string(),
            content: event.to_string(),
        };
        let serialized_message = serde_json::to_string(&message).unwrap();
        ctx.text(serialized_message); // Send the message to the client
    }
}

#[derive(Message)]
#[rtype(result = "()")] // No return value
pub struct ClientMessage(pub String);

impl Handler<ClientMessage> for MyWebSocket {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) {
        // Send the message content to the client
        ctx.text(msg.0);
    }
}

abigen!(
    FactoryV2Contract,
    r#"[{"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"type":"constructor"},{"constant":true,"inputs":[],"name":"INIT_CODE_PAIR_HASH","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"internalType":"uint256","name":"","type":"uint256"}],"name":"allPairs","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"allPairsLength","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"}],"name":"createPair","outputs":[{"internalType":"address","name":"pair","type":"address"}],"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"feeTo","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"feeToSetter","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"internalType":"address","name":"","type":"address"},{"internalType":"address","name":"","type":"address"}],"name":"getPair","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"_feeTo","type":"address"}],"name":"setFeeTo","outputs":[],"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"name":"setFeeToSetter","outputs":[],"stateMutability":"nonpayable","type":"function"},{"anonymous":false,"inputs":[{"indexed":true,"name":"token0","type":"address"},{"indexed":true,"name":"token1","type":"address"},{"indexed":false,"name":"pair","type":"address"},{"indexed":false,"name":"","type":"uint256"}],"name":"PairCreated","type":"event"}]"#
);

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    // Handle incoming WebSocket messages
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            // Handle incoming Text messages
            Ok(ws::Message::Text(text)) => {
                let request = serde_json::from_str::<MyMessage>(&text).unwrap();
                self.send_event(ctx, &format!("Server received: {}", text));

                if request.msg_type == "handshake" {
                    let payload = serde_json::json!({
                        "msg_type": "handshake",
                        "content": {
                            "status": true
                        }
                    });
                    tokio::spawn(async move {
                        send_to_client(payload.to_string(), false).await;
                    });
                }
                if request.msg_type == "Start sniping" {
                    let mut bot = Bot::new();

                    self.send_event(ctx, "pong");
                    let settings = serde_json::from_str::<SnipeSettings>(&request.content).unwrap();

                    tokio::spawn(async move {
                        bot.await.start_pool_snipe(settings).await.unwrap();
                    });
                }
                if request.msg_type == "swap" {
                    tokio::spawn(async move {
                        let data = serde_json::from_str::<SwapParams>(&request.content).unwrap();
                        let swap_message_queue = SWAP_MESSAGE_QUEUE.lock().await;
                        swap_message_queue.try_send(data).unwrap();
                    });
                }
                if request.msg_type == "close" {
                    println!("Closing connection as requested by the client.");
                    ctx.close(Some(ws::CloseReason {
                        code: ws::CloseCode::Normal,
                        description: Some("Closed by server".into()),
                    }));
                    tokio::spawn(async move {
                        let socket = WEBSOCKET_ADDR.lock().await.clone().unwrap();
                    });
                }
            }
            // Handle incoming Ping messages
            Ok(ws::Message::Ping(msg)) => {
                println!("Received ping: {:?}", msg);
                // Send pong in response
                ctx.pong(&msg);
            }
            // Handle incoming Close messages
            Ok(ws::Message::Close(reason)) => {
                println!("Client disconnected: {:?}", reason);
                ctx.stop();
            }
            // Handle unexpected message types
            _ => (),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockInfo {
    pub number: U64,
    pub timestamp: U256,
    pub base_fee: U256,
}

impl BlockInfo {
    // Create a new `BlockInfo` instance
    pub fn new(number: U64, timestamp: U256, base_fee: U256) -> Self {
        Self {
            number,
            timestamp,
            base_fee,
        }
    }

    #[allow(dead_code)]
    // Find the next block ahead of `prev_block`
    pub fn find_next_block_info(prev_block: Block<TxHash>) -> Self {
        let number = prev_block.number.unwrap_or_default() + 1;
        let timestamp = prev_block.timestamp + 12;
        let base_fee = calculate_next_block_base_fee(prev_block);
        Self {
            number,
            timestamp,
            base_fee,
        }
    }
}

pub struct Bot {
    token_sniper: TokenSniper,
    pool_list: Arc<Mutex<Vec<Value>>>,
}

impl Bot {
    pub async fn new(// web_socket_addr: Addr<MyWebSocket>
    ) -> Self {
        let env = Env::new();
        let mut token_sniper = TokenSniper::new();
        let new_pool_sender = broadcast::channel::<NewPoolEvent>(1000);
        let new_pool_receiver = new_pool_sender.0.subscribe();
        let new_token_sender = broadcast::channel::<NewTokenEvent>(1000);
        let new_token_receiver = new_token_sender.0.subscribe();

        let sender = env
            .private_key
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(369_u64);

        Self {
            token_sniper,
            pool_list: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn send_message_to_client(&self, message: &str) {
        // Send message to client using the stored websocket address
        // self.web_socket_addr.do_send(message);
    }

    pub async fn token_snipe(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut token_sniper = &mut self.token_sniper;
        let token_minted_event_signature = H256::from_str(
            &*"0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
        )?;

        let filter = Filter::new()
            .topic0(Topic::from(token_minted_event_signature))
            .topic1(Topic::from(Address::zero()));

        let client = create_local_client().await.clone();

        let mut log_stream = client.subscribe_logs(&filter).await?;
        // let new_token_sender = self.new_token_sender.0.clone();
        // let new_token_receiver = new_token_sender.subscribe();

        // token_sniper.snipe(new_token_receiver);
        while let Some(log) = log_stream.next().await {
            // info!("token logs => {:?}", log);
            let mut token = Token::new(
                Address::zero(),
                U256::zero(),
                U256::from(18_u8),
                "".parse().unwrap(),
            );

            token.address = log.address;
            token.total_supply = U256::from_big_endian(&log.data);

            // new_token_sender.send(NewTokenEvent::NewToken {
            //     token
            // }).expect("TODO: panic message");
        }
        Ok(())
    }

    pub async fn start_pool_snipe(
        &mut self,
        snipe_settings: SnipeSettings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pool_sniper_settings = snipe_settings.clone();

        println!("Starting pool snipe");
        let start_get_pool_message = serde_json::json!({
            "type": "start getting pool",
            "data": "started"
        });
        send_to_client(start_get_pool_message.to_string(), true).await;

        let pool_created_event_signature = H256::from_str(
            &*"0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9".to_string(),
        )?;

        // let new_pool_sender = self.new_pool_sender.0.clone();
        // let new_pool_receiver = new_pool_sender.subscribe();

        let filters = Filter::new()
            // .address(vec![*PULSEX_FACTORY_ADDRESS])
            .topic0(Topic::from(pool_created_event_signature));
        // .topic1(Topic::from(token_addresses));
        let client = create_local_client().await.clone();
        let mut log_stream = client.subscribe_logs(&filters).await?;
        let pool_list = Arc::clone(&self.pool_list);
        // self.pool_sniper.snipe(new_pool_receiver).await;

        while let Some(log) = log_stream.next().await {
            // info!("logs => {:?}", log);
            let mut pool = Pool::new(
                Address::random(),
                Address::random(),
                Address::random(),
                U256::zero(),
                2_u64,
            );
            let factory_address = log.address;
            let version = if factory_address == *PULSEX_FACTORY_ADDRESS_V1 {
                1
            } else if factory_address == *PULSEX_FACTORY_ADDRESS_V2 {
                2
            } else {
                0
            };

            let token0 = Address::from(log.topics[1]);
            let token1 = Address::from(log.topics[2]);
            let pair_address = Address::from_str(&*hex::encode(&log.data[12..32]))?;
            pool.address = pair_address;
            pool.token_0 = token0;
            pool.token_1 = token1;

            println!(
                "new pool: {} {} {} {}",
                pair_address, token0, token1, pool_sniper_settings.target_token
            );

            if (token0 != Address::from_str(&*pool_sniper_settings.target_token).unwrap()
                && token1 != Address::from_str(&*pool_sniper_settings.target_token).unwrap())
                || (token0 != Address::from_str(&*pool_sniper_settings.currency_token).unwrap()
                    && token1 != Address::from_str(&*pool_sniper_settings.currency_token).unwrap())
            {
                println!(
                    "Pool didn't contains {:?} {}",
                    pool, pool_sniper_settings.target_token
                );
                continue;
            }

            {
                let mut list = pool_list.lock().await;
                list.push(pool.to_value());
            }

            let serialized_pools = {
                let list = pool_list.lock().await;
                serde_json::to_string(&*list)?
            };

            send_to_client(serialized_pools, true).await;
            pool.address = pair_address;

            tokio::spawn({
                let snipe_settings = snipe_settings.clone();
                async move {
                    let pool_sniper = PoolSniper::new(pair_address, snipe_settings);
                    loop {
                        let snipe_receipt = pool_sniper.snipe(token0, token1, version).await;
                        let status = snipe_receipt.status.unwrap_or(U64::zero());
                        println!("status: {}", status);
                        if status == U64::one() {
                            break;
                        }

                        sleep(Duration::from_secs(1)).await;

                        let retry_message = serde_json::json!({
                            "msg_type": "retrying",
                            "content": {
                                "pair_address": pair_address,
                            }
                        });

                        send_to_client(retry_message.to_string(), true).await;
                    }
                }
            });

            info!("pool => {:?}", pool.clone());
        }

        Ok(())
    }
    pub async fn direct_swap(
        &mut self,
        pool: Pool,
        snipe_settings: SnipeSettings,
    ) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
        let pool_sniper = PoolSniper::new(pool.address, snipe_settings);
        let receipt = pool_sniper
            .snipe(pool.token_0, pool.token_1, pool.version)
            .await;
        // let status = receipt.status.unwrap_or(U64::zero());

        Ok(receipt)
    }

    pub async fn pool_snipe(
        &mut self,
        currency: String,
        first_token_address: String,
        token_to_buy: String,
        buy_amount: u64,
    ) -> U64 {
        let settings = Settings::new();
        let env = Env::new();

        let sender = env
            .private_key
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(369_u64);
        let client = create_local_client().await.clone();

        let slippage = settings.slippage_tolerance;
        let gas_price = settings.gas_price;

        let mut path = vec![
            Address::from_str(&*currency.clone()).unwrap(),
            Address::from_str(&*first_token_address.clone()).unwrap(),
            Address::from_str(&*token_to_buy.clone()).unwrap(),
        ];
        if Address::from_str(&*currency.clone()).unwrap()
            == Address::from_str(&*first_token_address.clone()).unwrap()
        {
            path = vec![
                Address::from_str(&*currency).unwrap(),
                Address::from_str(&*token_to_buy.clone()).unwrap(),
            ];
        }
        let nonce = client
            .get_transaction_count(sender.address(), None)
            .await
            .unwrap();

        let result = trade_swap_v2(
            path,
            U256::from(buy_amount),
            U256::one(),
            gas_price,
            slippage as f32,
            nonce,
        )
        .await;
        result.status.unwrap_or(U64::zero())
    }

    // pub async fn mempool_snipe(&mut self, token_addresses: Vec<Address>) -> Result<(), Box<dyn std::error::Error>> {
    //
    // while let Some(tx_hash) = mempool_stream.next().await {
    //     println!("Pending Transaction Hash: {:?}", tx_hash);
    //
    //     // Fetch the transaction details
    //     if let Ok(Some(tx)) = self.client.get_transaction(tx_hash).await {
    //         tx_sender.send(MemPoolEvent::NewTx { tx: tx.clone() })
    //             .expect("Failed to send new block through channel");
    //         print!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$ => {:?}", tx.clone());
    //
    //     } else {
    //         eprintln!("Failed to fetch transaction details for {:?}", tx_hash);
    //     }
    // }
    // }

    fn print_transaction_details(&self, tx: &Transaction) {
        println!("Transaction Details:");
        println!("From: {:?}", tx.from);
        println!("To: {:?}", tx.to);
        println!("Value: {:?}", tx.value);
        println!("Gas: {:?}", tx.gas);
        println!("Input Data: {:?}", tx.input);
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
