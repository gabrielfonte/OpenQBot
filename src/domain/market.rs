use serde_json::Value;

use crate::broker::account::KlineInterval;

#[derive(Clone)]
pub struct KlineEvent {
    pub symbol: String,
    pub interval: KlineInterval,
    pub close: f64,
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
