#![allow(dead_code)]

use serde_json::Value;

use crate::broker::account::KlineInterval;

#[derive(Clone)]
pub struct KlineEvent {
    pub symbol: String,
    pub interval: KlineInterval,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
}

#[derive(Clone)]
pub struct TradeEvent {
    pub symbol: String,
    pub price: Option<f64>,
    pub quantity: Option<f64>,
}

#[derive(Clone)]
pub struct AggregateTradeEvent {
    pub symbol: String,
    pub price: Option<f64>,
    pub quantity: Option<f64>,
}

#[derive(Clone)]
pub struct AveragePriceEvent {
    pub symbol: String,
    pub average_price: Option<f64>,
}

#[derive(Clone)]
pub enum MarketEvent {
    KLine(KlineEvent),
    Trade(TradeEvent),
    AggregateTrade(AggregateTradeEvent),
    AveragePrice(AveragePriceEvent),
    Unknown(Value),
}
