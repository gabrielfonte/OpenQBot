use super::ws;
use super::mapper;
use fastwebsockets::Frame;
use fastwebsockets::OpCode;
use serde_json::Value;
use crate::broker::stream::{EventAndSymbol, Publisher, StreamProvider, Subscriber, SubscriberId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

type ShutdownTx = oneshot::Sender<()>;

pub struct BinanceStreamProvider {
    domain:    String,
    publisher: Arc<Mutex<Publisher>>,
    tasks:     HashMap<SubscriberId, (JoinHandle<()>, ShutdownTx)>,
}

impl BinanceStreamProvider {
    pub fn new(domain: &str) -> Self {
        Self {
            domain:    domain.to_string(),
            publisher: Arc::new(Mutex::new(Publisher::default())),
            tasks:     HashMap::new(),
        }
    }
}

impl StreamProvider for BinanceStreamProvider {
    fn subscribe(&mut self, event: EventAndSymbol, listener: Subscriber) -> SubscriberId {
        let id = self.publisher.lock().unwrap().subscribe(event.clone(), listener);

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        let domain    = self.domain.clone();
        let endpoint  = match event {
            EventAndSymbol::KLine(ref symbol, ref interval) => format!("{}@kline_{}", symbol, interval),
            EventAndSymbol::Trade(ref symbol) => format!("{}@trade", symbol),
            EventAndSymbol::AggregateTrade(ref symbol) => format!("{}@aggTrade", symbol),
            EventAndSymbol::AveragePrice(ref symbol) => format!("{}@avgPrice", symbol),
        };
        let publisher = Arc::clone(&self.publisher);
        let event_key = event.clone();

        let handle = tokio::spawn(async move {
            let ws = match ws::connect(&domain, &("ws/".to_owned() + &endpoint)).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("WS connect error for {}: {}", endpoint, e);
                    return;
                }
            };

            let mut ws = ws;

            loop {tokio::select! {
                    msg = ws.read_frame() => {
                        match msg {
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
                                    let event_value = match mapper::to_market_event(&event_key, value) {
                                        Some(v) => v,
                                        None => continue,
                                    };
                                    publisher.lock().unwrap().notify(&event_key, event_value);
                                }
                                OpCode::Close => {
                                    eprintln!("WS closed for {}", endpoint);
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

                    _ = &mut shutdown_rx => {
                        let _ = ws.write_frame(Frame::close_raw(vec![].into())).await;
                        break;
                    }
                }
            }
        });

        self.tasks.insert(id, (handle, shutdown_tx));
        id
    }

    fn unsubscribe(&mut self, event: EventAndSymbol, id: SubscriberId) {
        if let Some((handle, shutdown_tx)) = self.tasks.remove(&id) {
            let _ = shutdown_tx.send(());
            handle.abort();
        }
        self.publisher.lock().unwrap().unsubscribe(event, id);
    }
}