extern crate dotenv;

use std::sync::Arc;
use crate::broker::account::{Account, BalanceType, KlineInterval};
use std::time;
use dotenv::dotenv;
use tokio::sync::RwLock;
use broker::binance::account;
use crate::broker::binance::stream::BinanceStreamProvider;
use crate::broker::kraken::stream::KrakenStreamProvider;
use crate::broker::stream::{EventAndSymbol, StreamProvider};
use crate::domain::market::MarketEvent;
use crate::strategies::strategy::TradingStrategy;

mod broker;
mod domain;
mod indicators;
mod strategies;

// This is not a final design, just a quick test to see if the pieces fit together. 
// The main function will eventually be replaced by a more robust application structure with proper error handling, configuration management, etc.

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ()>{
    dotenv().ok();

    let acc = account::BinanceAccount::new();

    let balance = acc.unwrap().get_account_balance(BalanceType::Spot, "BRL").await.unwrap();
    println!("{:?}", balance);

    let mut stream = BinanceStreamProvider::new("stream.binance.com");
    let mut kraken_stream = KrakenStreamProvider::new("ws.kraken.com");

    // Order book updates for BTC/USDT
    stream.subscribe(
        EventAndSymbol::Trade("btcusdt".to_string()),
        Arc::new(|event: MarketEvent| {
            if let MarketEvent::Trade(trade) = event {
                let _ = trade.price;
            }
        }),
    );

    // Kline updates for BTC/USDT 1m on Kraken
    kraken_stream.subscribe(
        EventAndSymbol::KLine("BTC/USD".to_string(), KlineInterval::OneMinute),
        Arc::new(|event: MarketEvent| {
            if let MarketEvent::KLine(kline) = event {
                let _ = kline.close;
                println!("Kraken Kline: {} close price {}", kline.symbol, kline.close);
            }
        }),
    );

    // Test Bollinger Bands strategy
    let bollinger_bands_strategy = Arc::new(RwLock::new(strategies::bollinger_bounce::BollingerBounceTradingStrategy::new(20, 2.0)));

    let sub_id = stream.subscribe(
        EventAndSymbol::KLine("btcusdt".to_string(), KlineInterval::OneSecond),
        Arc::new({
            move |event: MarketEvent| {
                let lock = Arc::clone(&bollinger_bands_strategy);
                tokio::spawn(async move {
                    let mut bollinger_bands = lock.write().await;
                    bollinger_bands.on_event(&event);
                });
            }
        }),
    );

    tokio::time::sleep(time::Duration::from_secs(600)).await;
    stream.unsubscribe(EventAndSymbol::KLine("btcusdt".to_string(), KlineInterval::OneSecond), sub_id);

    Ok(())
}
