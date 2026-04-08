#![allow(dead_code)]

use crate::broker::ws;
use super::mapper;
use fastwebsockets::{Frame, OpCode, Payload};
use serde_json::{json, Value};
use crate::domain::market::KlineInterval;
use crate::broker::stream::{EventAndSymbol, Publisher, StreamProvider, Subscriber, SubscriberId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct KrakenStreamProvider {
    publisher: Arc<Mutex<Publisher>>,
    stream_map: Arc<Mutex<HashMap<String, EventAndSymbol>>>,
    cmd_tx: mpsc::Sender<WsCommand>,
    _task: JoinHandle<()>,
    refcounts: HashMap<String, usize>,
}

enum WsCommand {
    Subscribe(EventAndSymbol),
    Unsubscribe(EventAndSymbol),
}

fn event_to_channel(event: &EventAndSymbol) -> &'static str {
    match event {
        EventAndSymbol::KLine(_, _) => "ohlc",
        EventAndSymbol::Trade(_) => "ticker",
        EventAndSymbol::AggregateTrade(_) => "ticker",
        EventAndSymbol::AveragePrice(_) => "ticker",
    }
}

fn event_to_stream_key(event: &EventAndSymbol) -> String {
    match event {
        EventAndSymbol::KLine(symbol, interval) => {
            format!(
                "{}:{}:{}",
                event_to_channel(event),
                symbol,
                kraken_interval_minutes(interval).unwrap()
            )
        }
        EventAndSymbol::Trade(symbol)
        | EventAndSymbol::AggregateTrade(symbol)
        | EventAndSymbol::AveragePrice(symbol) => {
            format!("{}:{}", event_to_channel(event), symbol)
        }
    }
}

fn subscription_payload(method: &str, event: &EventAndSymbol) -> String {
    match event {
        EventAndSymbol::KLine(symbol, interval) => json!({
            "method": method,
            "params": {
                "channel": "ohlc",
                "symbol": [symbol],
                "interval": kraken_interval_minutes(interval),
            }
        })
        .to_string(),
        EventAndSymbol::Trade(symbol)
        | EventAndSymbol::AggregateTrade(symbol)
        | EventAndSymbol::AveragePrice(symbol) => json!({
            "method": method,
            "params": {
                "channel": "ticker",
                "symbol": [symbol],
            }
        })
        .to_string(),
    }
}

fn kraken_interval_minutes(interval: &KlineInterval) -> Option<u64> {
    match interval {
        KlineInterval::OneMinute => Some(1),
        KlineInterval::FiveMinutes => Some(5),
        KlineInterval::FifteenMinutes => Some(15),
        KlineInterval::ThirtyMinutes => Some(30),
        KlineInterval::OneHour => Some(60),
        KlineInterval::FourHours => Some(240),
        KlineInterval::OneDay => Some(1440),
        KlineInterval::OneWeek => Some(10080),
        _ => None,
    }
}

fn value_as_string(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        return Some(s.to_string());
    }
    if let Some(i) = v.as_i64() {
        return Some(i.to_string());
    }
    if let Some(f) = v.as_f64() {
        return Some(f.to_string());
    }
    None
}

fn stream_key_from_message(value: &Value) -> Option<String> {
    let channel = value["channel"].as_str()?;

    let first_data = value
        .get("data")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first());

    let symbol = first_data
        .and_then(|d| d.get("symbol"))
        .and_then(value_as_string)
        .or_else(|| value.get("symbol").and_then(value_as_string));

    let interval = first_data
        .and_then(|d| d.get("interval"))
        .and_then(value_as_string)
        .or_else(|| value.get("interval").and_then(value_as_string));

    match (channel, symbol, interval) {
        ("ohlc", Some(sym), Some(intv)) => Some(format!("{}:{}:{}", channel, sym, intv)),
        (_, Some(sym), _) => Some(format!("{}:{}", channel, sym)),
        _ => None,
    }
}

impl KrakenStreamProvider {
    pub fn new(domain: &str) -> Self {
        let publisher: Arc<Mutex<Publisher>> = Arc::new(Mutex::new(Publisher::default()));
        let stream_map: Arc<Mutex<HashMap<String, EventAndSymbol>>> = Arc::new(Mutex::new(HashMap::new()));
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<WsCommand>(64);

        let publisher_clone = Arc::clone(&publisher);
        let stream_map_clone = Arc::clone(&stream_map);
        let domain = domain.to_string();

        let task = tokio::spawn(async move {
            let mut ws = match ws::connect(&domain, "v2").await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("WS connect error: {}", e);
                    return;
                }
            };


            loop {
                tokio::select! {
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(WsCommand::Subscribe(event)) => {
                                let msg = subscription_payload("subscribe", &event);
                                let _ = ws.write_frame(Frame::text(Payload::from(msg.as_bytes()))).await;
                            }
                            Some(WsCommand::Unsubscribe(event)) => {
                                let msg = subscription_payload("unsubscribe", &event);
                                let _ = ws.write_frame(Frame::text(Payload::from(msg.as_bytes()))).await;
                            }
                            None => {
                                let _ = ws.write_frame(Frame::close_raw(vec![].into())).await;
                                break;
                            }
                        }
                    }
                    frame = ws.read_frame() => {
                        match frame {
                            Ok(frame) => match frame.opcode {
                                OpCode::Text => {
                                    let payload = match String::from_utf8(frame.payload.to_vec()) {
                                        Ok(p) => p,
                                        Err(_) => continue,
                                    };
                                    let value: Value = match serde_json::from_str(&payload) {
                                        Ok(v) => v,
                                        Err(_) => continue,
                                    };
                                    let stream_key = match stream_key_from_message(&value) {
                                        Some(k) => k,
                                        None    => continue,
                                    };
                                    let event_key = match stream_map_clone.lock().unwrap().get(&stream_key).cloned() {
                                        Some(k) => k,
                                        None    => continue,
                                    };
                                    let data = value["data"].clone();
                                    let market_event = match mapper::to_market_event(&event_key, data) {
                                        Some(e) => e,
                                        None    => continue,
                                    };
                                    publisher_clone.lock().unwrap().notify(&event_key, market_event);
                                }
                                OpCode::Close => {
                                    eprintln!("WS closed");
                                    break;
                                }
                                _ => {}
                            },
                            Err(e) => {
                                eprintln!("WS read error: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });

        Self {
            publisher,
            stream_map,
            cmd_tx,
            _task: task,
            refcounts: HashMap::new(),
        }
    }
}

impl StreamProvider for KrakenStreamProvider {
    fn subscribe(&mut self, event: EventAndSymbol, listener: Subscriber) -> SubscriberId {
       let stream_key = event_to_stream_key(&event);
       let id = self.publisher.lock().unwrap().subscribe(event.clone(), listener);
       let count = self.refcounts.entry(stream_key.clone()).or_insert(0);
       *count += 1;
       if *count == 1 {
            self.stream_map.lock().unwrap().insert(stream_key, event.clone());
            if let Err(e) = self.cmd_tx.try_send(WsCommand::Subscribe(event)) {
                eprintln!("Failed to send WS subscribe command: {}", e);
            }
        }
        
       id
    }

    fn unsubscribe(&mut self, event: EventAndSymbol, id: SubscriberId) {
        let stream_key = event_to_stream_key(&event);
        self.publisher.lock().unwrap().unsubscribe(event.clone(), id);

        if let Some(count) = self.refcounts.get_mut(&stream_key) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.refcounts.remove(&stream_key);
                self.stream_map.lock().unwrap().remove(&stream_key);
                if let Err(e) = self.cmd_tx.try_send(WsCommand::Unsubscribe(event)) {
                    eprintln!("Failed to send WS unsubscribe command: {}", e);
                }
            }
        }
    }
}
