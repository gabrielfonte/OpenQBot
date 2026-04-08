use crate::broker::ws;
use super::mapper;
use fastwebsockets::{Frame, OpCode, Payload};
use serde_json::{json, Value};
use crate::broker::stream::{EventAndSymbol, Publisher, StreamProvider, Subscriber, SubscriberId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

enum WsCommand {
    Subscribe(String),
    Unsubscribe(String),
}

pub struct BinanceStreamProvider {
    publisher:  Arc<Mutex<Publisher>>,
    stream_map: Arc<Mutex<HashMap<String, EventAndSymbol>>>,
    cmd_tx:     mpsc::Sender<WsCommand>,
    _task:      JoinHandle<()>,
    refcounts:  HashMap<String, usize>,
}

fn event_to_stream_name(event: &EventAndSymbol) -> String {
    match event {
        EventAndSymbol::KLine(symbol, interval)  => format!("{}@kline_{}", symbol, interval),
        EventAndSymbol::Trade(symbol)             => format!("{}@trade", symbol),
        EventAndSymbol::AggregateTrade(symbol)    => format!("{}@aggTrade", symbol),
        EventAndSymbol::AveragePrice(symbol)      => format!("{}@avgPrice", symbol),
    }
}

impl BinanceStreamProvider {
    pub fn new(domain: &str) -> Self {
        let publisher:  Arc<Mutex<Publisher>>                     = Arc::new(Mutex::new(Publisher::default()));
        let stream_map: Arc<Mutex<HashMap<String, EventAndSymbol>>> = Arc::new(Mutex::new(HashMap::new()));
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<WsCommand>(64);

        let publisher_clone  = Arc::clone(&publisher);
        let stream_map_clone = Arc::clone(&stream_map);
        let domain           = domain.to_string();

        let task = tokio::spawn(async move {
            let mut ws = match ws::connect(&domain, "stream").await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("WS connect error: {}", e);
                    return;
                }
            };

            let mut req_id: u64 = 1;

            loop {
                tokio::select! {
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(WsCommand::Subscribe(stream)) => {
                                let msg = json!({
                                    "method": "SUBSCRIBE",
                                    "params": [stream],
                                    "id": req_id,
                                }).to_string();
                                req_id += 1;
                                let _ = ws.write_frame(Frame::text(Payload::from(msg.as_bytes()))).await;
                            }
                            Some(WsCommand::Unsubscribe(stream)) => {
                                let msg = json!({
                                    "method": "UNSUBSCRIBE",
                                    "params": [stream],
                                    "id": req_id,
                                }).to_string();
                                req_id += 1;
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
                                    let stream_name = match value["stream"].as_str() {
                                        Some(s) => s.to_string(),
                                        None    => continue,
                                    };
                                    let event_key = match stream_map_clone.lock().unwrap().get(&stream_name).cloned() {
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

impl StreamProvider for BinanceStreamProvider {
    fn subscribe(&mut self, event: EventAndSymbol, listener: Subscriber) -> SubscriberId {
        let stream_name = event_to_stream_name(&event);
        let id          = self.publisher.lock().unwrap().subscribe(event.clone(), listener);

        let count = self.refcounts.entry(stream_name.clone()).or_insert(0);
        *count += 1;
        if *count == 1 {
            self.stream_map.lock().unwrap().insert(stream_name.clone(), event);
            if let Err(e) = self.cmd_tx.try_send(WsCommand::Subscribe(stream_name)) {
                eprintln!("Failed to send WS subscribe command: {}", e);
            }
        }

        id
    }

    fn unsubscribe(&mut self, event: EventAndSymbol, id: SubscriberId) {
        let stream_name = event_to_stream_name(&event);
        self.publisher.lock().unwrap().unsubscribe(event, id);

        if let Some(count) = self.refcounts.get_mut(&stream_name) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.refcounts.remove(&stream_name);
                self.stream_map.lock().unwrap().remove(&stream_name);
                if let Err(e) = self.cmd_tx.try_send(WsCommand::Unsubscribe(stream_name)) {
                    eprintln!("Failed to send WS unsubscribe command: {}", e);
                }
            }
        }
    }
}