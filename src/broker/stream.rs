use std::collections::HashMap;
use std::sync::Arc;
use crate::broker::account::KlineInterval;
use crate::domain::market::MarketEvent;

pub type SubscriberId = u64;
pub type Subscriber = Arc<dyn Fn(MarketEvent) + Send + Sync>;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum EventAndSymbol {
    KLine(String, KlineInterval),
    Trade(String),
    AggregateTrade(String),
    AveragePrice(String),
}

pub trait StreamProvider: Send + Sync {
    fn subscribe(&mut self, event: EventAndSymbol, listener: Subscriber) -> SubscriberId;
    fn unsubscribe(&mut self, event: EventAndSymbol, id: SubscriberId);
}

#[derive(Default)]
pub struct Publisher {
    events: HashMap<EventAndSymbol, Vec<(SubscriberId, Subscriber)>>,
    next_id: SubscriberId,
}

impl Publisher {
    pub fn subscribe(&mut self, event: EventAndSymbol, listener: Subscriber) -> SubscriberId {
        let id = self.next_id;
        self.next_id += 1;
        self.events.entry(event).or_default().push((id, listener));
        id
    }

    pub fn unsubscribe(&mut self, event: EventAndSymbol, id: SubscriberId) {
        if let Some(listeners) = self.events.get_mut(&event) {
            listeners.retain(|(sid, _)| *sid != id);
        }
    }

    pub fn notify(&self, event: &EventAndSymbol, value: MarketEvent) {
        if let Some(listeners) = self.events.get(event) {
            for (_, listener) in listeners {
                listener(value.clone());
            }
        }
    }
}