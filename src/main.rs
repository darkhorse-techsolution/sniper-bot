mod bot;
mod token_sniper;
mod constants;
mod pool_sniper;
mod abi;
mod transaction_simulator;
mod utils;

use std::fmt::format;
use std::str::FromStr;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc};
use actix::{Addr};
use actix_cors::Cors;
use actix_web::web::Json;
use actix_web_actors::ws;
use actix_web_actors::ws::WsResponseBuilder;
use ethers::prelude::abigen;
use tokio::sync::Mutex;
use ethers::types::Address;
use ethers_core::types::U256;
use serde_json::{json, Value};
use bot::Bot;
use crate::bot::{ClientMessage, MyWebSocket};
use crate::constants::{process_swap_queue, Env, SwapHandler, PULSEX_SWAP_ROUTER, SWAP_MESSAGE_QUEUE, WEBSOCKET_ADDR, WEB_HOOK_URL};
use crate::utils::helpers::{create_local_client, setup_websocket_client, setup_logger, get_allowance, approve};
use crate::utils::types::structs::setting::{Settings, SnipeSettings};
use crate::utils::types::structs::token::Token;
use tokio::sync::{mpsc};
use tokio::sync::mpsc::Receiver;
use crate::utils::types::structs::pool::Pool;
use crate::utils::types::structs::setting::SwapParams;

#[derive(Serialize)]
struct Response {
    message: String,
    value: Option<Value>,
}

#[derive(Deserialize, Serialize, Debug)]
struct PoolSnipingRequest {
    poolAddress: String,
    currency: String,
    tokenToBuy: String,
    firstToken: String,
    buyAmount: String,
}

#[derive(Serialize)]
struct TokenResponse {
    message: String,
    value: Settings,
}

struct AppState {
    bot: Arc<Mutex<Bot>>,
}
async fn token_snipe(data: web::Data<AppState>) -> impl Responder {
    dotenv::dotenv().ok();

    let mut bot = data.bot.lock().await;

    match bot.token_snipe().await {
        Ok(_) => HttpResponse::Ok().json(Response {
            message: String::from("Bot started successfully!"),
            value: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(Response {
            message: format!("Failed to start bot: {}", e),
            value: None,
        }),
    }
}
async fn start_snipe_pool(data: web::Data<AppState>) -> impl Responder {
    dotenv::dotenv().ok();

    // bot.start_pool_snipe(data).await.unwrap();
    HttpResponse::Ok().json(Response {
        message: String::from("Bot started successfully!"),
        value: None,
    })
}

async fn stop_snipe_pool(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(Response {
        message: String::from("Bot Stopped!"),
        value: None,
    })
}

async fn snipe_pool(data: web::Json<PoolSnipingRequest>) -> impl Responder {
    dotenv::dotenv().ok();
    println!("Response data => {:?}", data);
    // let target_tokens =

    // Lock the Mutex to get mutable access to the bot
    // bot.pool_snipe();
    let mut bot = Bot::new().await;
    let status = bot.pool_snipe(data.currency.clone(), data.firstToken.clone(), data.tokenToBuy.clone(), data.buyAmount.clone().parse().unwrap()).await;
    HttpResponse::Ok().json(Response {
        message: String::from("successfully!"),
        value: None,
    })
}


async fn stop_bot(data: web::Data<AppState>) -> impl Responder {
    let mut bot = data.bot.lock().await;
    match bot.stop().await {
        Ok(_) => HttpResponse::Ok().json(Response {
            message: String::from("Bot stopped successfully!"),
            value: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(Response {
            message: format!("Failed to stop bot: {}", e),
            value: None,
        }),
    }
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(Response {
        message: String::from("Server is running!"),
        value: None,
    })
}

async fn get_tokens() -> impl Responder {
    let settings = Settings::new();
    print!("settings: {:?}", settings);
    HttpResponse::Ok().json(TokenResponse {
        message: String::from("Tokens received"),
        value: settings,
    })
}

type SharedAddr = Arc<Mutex<Option<Addr<MyWebSocket>>>>;

async fn websocket_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let ws_response_builder = WsResponseBuilder::new(MyWebSocket, &req, stream);
    let (addr, res) = ws_response_builder.start_with_addr()?;

    {
        let mut shared_addr = WEBSOCKET_ADDR.lock().await;
        *shared_addr = Some(addr.clone());
    }
    let shake_message = serde_json::json!({
        "type": "start",
        "data": "client connected"
    });

    addr.do_send(ClientMessage(shake_message.to_string()));
    // Now you have the `Addr<MyWebSocket>` instance in `addr`
    println!("WebSocket actor started: {:?} {:?}", addr, res);

    Ok(res)
}

async fn process_swaps(mut receiver: Receiver<SwapParams>) {
    while let Some(swap_params) = receiver.recv().await {
        // Perform the swap operation
        // For example, call the direct_swap function
        println!("Processing swap: {:?}", swap_params);

        // Replace this with your direct_swap implementation
        tokio::spawn(async move {
            let mut bot = Bot::new().await;
            let client = create_local_client().await;
            println!("Bot connected: {:?}", swap_params);
        });
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_logger().expect("Failed to set up logger");
    dotenv::dotenv().ok();
    let env = Env::new();

    println!("{}", format!("Server starting at http://127.0.0.1:{}", env.port));

    // websocket_addr.start();
    // Wrap Bot in a Mutex and then in an Arc
    let bot = Arc::new(Mutex::new(Bot::new().await));
    let app_state = web::Data::new(AppState { bot });

    let currency_address = Address::from_str("0x2b591e99afE9f32eAA6214f7B7629768c40Eeb39").unwrap();
    let allowance = get_allowance(currency_address).await;

    println!("Allowance: {}", allowance);
    if allowance < U256::from(100000000000000_u128) {
        println!("approving...");
        approve(currency_address, *PULSEX_SWAP_ROUTER, U256::max_value()).await;
    }

    // token_snipe(app_state.clone()).await;
    // pool_snipe(app_state.clone()).await;

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check)) // Added health check route
            .route("/bot/token_snipe", web::post().to(token_snipe))
            .route("/bot/start_snipe_pool", web::get().to(start_snipe_pool))
            .route("/bot/stop_snipe_pool", web::get().to(stop_snipe_pool))
            .route("bot/pool_snipe", web::post().to(snipe_pool))
            .route("/bot/stop", web::post().to(stop_bot))
            .route("/settings/get_tokens", web::get().to(get_tokens))
            .route("/ws", web::get().to(websocket_handler))
    })
        .bind(format!("127.0.0.1:{}", env.port).as_str())?
        .run()
        .await
}