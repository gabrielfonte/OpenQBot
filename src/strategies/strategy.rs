use crate::domain::market::MarketEvent;

pub trait TradingStrategy {
    fn on_event(&mut self, event: &MarketEvent);
}