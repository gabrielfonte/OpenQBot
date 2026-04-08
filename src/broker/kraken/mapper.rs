#![allow(dead_code)]

use serde_json::Value;

use crate::broker::stream::EventAndSymbol;
use crate::domain::market::{KlineEvent, MarketEvent};

fn get_field_from_payload(field: &str, value: &Value) -> Option<f64> {
    let first = value
        .as_array()
        .and_then(|arr| arr.first())
        .unwrap_or(value);

    first
        .get(field)
        .or_else(|| first.get(&field.chars().next()?.to_string()))
        .and_then(|v| {
            v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
        })
}

pub fn to_market_event(event: &EventAndSymbol, value: Value) -> Option<MarketEvent> {
    match event {
        EventAndSymbol::KLine(symbol, interval) => {
            println!("Kraken Stream: {}", value);

            let open = get_field_from_payload("open", &value)?;
            let close = get_field_from_payload("close", &value)?;
            let high = get_field_from_payload("high", &value)?;
            let low = get_field_from_payload("low", &value)?;
            let volume = get_field_from_payload("volume", &value)?;
            Some(MarketEvent::KLine(KlineEvent {
                symbol: symbol.clone(),
                interval: interval.clone(),
                open,
                close,
                high,
                low,
                volume
            }))
        }
        _ => None,
    }
}
