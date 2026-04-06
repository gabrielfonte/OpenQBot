use serde_json::Value;

use crate::broker::stream::EventAndSymbol;
use crate::domain::market::{
    AggregateTradeEvent, AveragePriceEvent, KlineEvent, MarketEvent, TradeEvent,
};

pub fn to_market_event(event: &EventAndSymbol, value: Value) -> Option<MarketEvent> {
    match event {
        EventAndSymbol::KLine(symbol, interval) => {
            let close = value["k"]["c"].as_str()?.parse::<f64>().ok()?;
            Some(MarketEvent::KLine(KlineEvent {
                symbol: symbol.clone(),
                interval: interval.clone(),
                close,
            }))
        }
        EventAndSymbol::Trade(symbol) => {
            let price = value["p"].as_str().and_then(|s| s.parse::<f64>().ok());
            let quantity = value["q"].as_str().and_then(|s| s.parse::<f64>().ok());
            Some(MarketEvent::Trade(TradeEvent {
                symbol: symbol.clone(),
                price,
                quantity,
            }))
        }
        EventAndSymbol::AggregateTrade(symbol) => {
            let price = value["p"].as_str().and_then(|s| s.parse::<f64>().ok());
            let quantity = value["q"].as_str().and_then(|s| s.parse::<f64>().ok());
            Some(MarketEvent::AggregateTrade(AggregateTradeEvent {
                symbol: symbol.clone(),
                price,
                quantity,
            }))
        }
        EventAndSymbol::AveragePrice(symbol) => {
            let average_price = value["w"].as_str().and_then(|s| s.parse::<f64>().ok());
            Some(MarketEvent::AveragePrice(AveragePriceEvent {
                symbol: symbol.clone(),
                average_price,
            }))
        }
    }
}
