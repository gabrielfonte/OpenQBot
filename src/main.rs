extern crate dotenv;
extern crate core;

use std::sync::Arc;
use crate::broker::account::{Account, BalanceType, KlineInterval};
use std::time;
use dotenv::dotenv;
use serde_json::Value;
use broker::binance::account;
use crate::broker::binance::stream::BinanceStreamProvider;
use crate::broker::stream::{EventAndSymbol, StreamProvider};

mod broker;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ()>{
    dotenv().ok();

    let acc = account::BinanceAccount::new();

    let balance = acc.unwrap().get_account_balance(BalanceType::Spot, "BRL").await.unwrap();
    println!("{:?}", balance);

    let mut stream = BinanceStreamProvider::new("stream.binance.com");

    // Get Kline data for BTC/USDT every second
    let id = stream.subscribe(
        EventAndSymbol::KLine("btcusdt".to_string(), KlineInterval::OneSecond),
        Arc::new(|value: Value| {
            println!("Kline received: {}", value);
        }),
    );

    // Order book updates for BTC/USDT
    stream.subscribe(
        EventAndSymbol::Trade("btcusdt".to_string()),
        Arc::new(|value: Value| {
            println!("Trade received: {}", value);
        }),
    );

    tokio::time::sleep(time::Duration::from_secs(10)).await;
    stream.unsubscribe(EventAndSymbol::KLine("btcusdt".to_string(), KlineInterval::OneSecond), id);

    Ok(())
}
