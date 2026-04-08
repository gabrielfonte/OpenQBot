extern crate dotenv;

use std::sync::Arc;
use crate::broker::account::{Account, BalanceType};
use std::time;
use dotenv::dotenv;
use tokio::sync::RwLock;
use broker::binance::account;
use crate::broker::binance::stream::BinanceStreamProvider;
use crate::broker::kraken::stream::KrakenStreamProvider;
use crate::broker::kraken::auth::Auth;
use crate::broker::stream::{EventAndSymbol, StreamProvider};
use crate::domain::market::{KlineInterval, MarketEvent};
use crate::strategies::strategy::TradingStrategy;

mod broker;
mod domain;
mod indicators;
mod strategies;

// This is not a final design, just a quick test to see if the pieces fit together. 
// The main function will eventually be replaced by a more robust application structure with proper error handling, configuration management, etc.

#[cfg(feature = "binance_demo")]
async fn binance_demo(mut stream: BinanceStreamProvider) -> Result<(), Box<dyn std::error::Error>> {
    let acc = account::BinanceAccount::new()?;
    let balance = acc.get_account_balance(BalanceType::Spot, "BRL").await?;
    println!("Binance Balance: {}", balance);

    // Order book updates for BTC/USDT
    stream.subscribe(
        EventAndSymbol::Trade("btcusdt".to_string()),
        Arc::new(|event: MarketEvent| {
            if let MarketEvent::Trade(trade) = event {
                println!("Binance Trade: {} price={} qty={}", trade.symbol, trade.price.unwrap(), trade.quantity.unwrap());
            }
        }),
    );

    // Test Bollinger Bounce strategy
    let bollinger_bounce_strategy = Arc::new(RwLock::new(strategies::bollinger_bounce::BollingerBounceTradingStrategy::new(20, 2.0)));

    stream.subscribe(
        EventAndSymbol::KLine("btcusdt".to_string(), KlineInterval::OneSecond),
        Arc::new({
            move |event: MarketEvent| {
                if let MarketEvent::KLine(kline) = &event {
                    println!("Binance Kline: {} close={} volume={}", kline.symbol, kline.close, kline.volume);
                }
                let lock = Arc::clone(&bollinger_bounce_strategy);
                tokio::spawn(async move {
                    let mut bollinger_bounce = lock.write().await;
                    bollinger_bounce.on_event(&event);
                });
            }
        }),
    );

    // Keep stream alive indefinitely
    loop {
        tokio::time::sleep(time::Duration::from_secs(60)).await;
    }
}

#[cfg(feature = "kraken_demo")]
async fn kraken_demo(mut kraken_stream: KrakenStreamProvider) -> Result<(), Box<dyn std::error::Error>> {
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

    // Authentication on Kraken and token refresh test
    let mut auth = Auth::new()?;
    auth.initialize()?;

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(time::Duration::from_secs(10)).await;
            match auth.get_token() {
                Ok(token) => println!("Current Kraken auth token: {}", token),
                Err(e) => eprintln!("Kraken auth token unavailable: {}", e),
            }
        }
    });

    
    // Test Bollinger Bounce strategy
    let bollinger_bounce_strategy = Arc::new(RwLock::new(strategies::bollinger_bounce::BollingerBounceTradingStrategy::new(20, 2.0)));

    kraken_stream.subscribe(
        EventAndSymbol::KLine("BTC/USD".to_string(), KlineInterval::OneMinute),
        Arc::new({
            move |event: MarketEvent| {
                let lock = Arc::clone(&bollinger_bounce_strategy);
                tokio::spawn(async move {
                    let mut bollinger_bounce = lock.write().await;
                    bollinger_bounce.on_event(&event);
                });
            }
        }),
    );


    // Keep stream alive indefinitely
    loop {
        tokio::time::sleep(time::Duration::from_secs(60)).await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ()>{
    dotenv().ok();

    #[cfg(feature = "binance_demo")]
    let binance_stream = BinanceStreamProvider::new("stream.binance.com");

    #[cfg(feature = "kraken_demo")]
    let kraken_stream = KrakenStreamProvider::new("ws.kraken.com");

    #[cfg(feature = "binance_demo")]
    tokio::spawn(async move {
        if let Err(e) = binance_demo(binance_stream).await {
            eprintln!("Binance demo failed: {}", e);
        }
    });

    #[cfg(feature = "kraken_demo")]
    tokio::spawn(async move {
        if let Err(e) = kraken_demo(kraken_stream).await {
            eprintln!("Kraken demo failed: {}", e);
        }
    });

    // Keep the runtime alive indefinitely
    loop {
        tokio::time::sleep(time::Duration::from_secs(60)).await;
    }
}
