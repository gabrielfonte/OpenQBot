extern crate dotenv;
extern crate core;

use std::sync::Arc;
use crate::broker::account::{Account, BalanceType, KlineInterval};
use std::time;
use dotenv::dotenv;
use hmac::Mac;
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use broker::binance::account;
use crate::broker::binance::stream::BinanceStreamProvider;
use crate::broker::stream::{EventAndSymbol, StreamProvider};
use crate::indicators::sma::SMA;
use crate::strategies::strategy::TradingStrategy;

mod broker;
mod indicators;
mod strategies;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ()>{
    dotenv().ok();

    let acc = account::BinanceAccount::new();

    let balance = acc.unwrap().get_account_balance(BalanceType::Spot, "BRL").await.unwrap();
    println!("{:?}", balance);

    let mut stream = BinanceStreamProvider::new("stream.binance.com");

    // Order book updates for BTC/USDT
    stream.subscribe(
        EventAndSymbol::Trade("btcusdt".to_string()),
        Arc::new(|value: Value| {
            //println!("Trade received: {}", value);
        }),
    );

    // Test Bollinger Bands strategy
    let bollinger_bands_strategy = Arc::new(RwLock::new(strategies::bollinger_bounce::BollingerBounceTradingStrategy::new(20, 2.0)));

    let sub_id = stream.subscribe(
        EventAndSymbol::KLine("btcusdt".to_string(), KlineInterval::OneSecond),
        Arc::new({
            move |value: Value| {
                let lock = Arc::clone(&bollinger_bands_strategy);
                tokio::spawn(async move {
                    //println!("Kline received: {}", value);
                    let close = value["k"]["c"]
                        .as_str()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    //println!("Close price: {}", close);
                    let mut bollinger_bands = lock.write().await;
                    bollinger_bands.on_kline("btcusdt", "1s", value);
                });
            }
        }),
    );

    tokio::time::sleep(time::Duration::from_secs(600)).await;
    stream.unsubscribe(EventAndSymbol::KLine("btcusdt".to_string(), KlineInterval::OneSecond), sub_id);

    Ok(())
}
