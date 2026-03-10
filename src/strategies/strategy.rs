use serde_json::Value;

pub trait TradingStrategy {
    fn on_kline(&mut self, symbol: &str, interval: &str, kline: Value); // TODO: use kline enum instead of str
    fn on_trade(&mut self, symbol: &str, trade: Value);
    fn on_aggregate_trade(&mut self, symbol: &str, agg_trade: Value);
    fn on_average_price(&mut self, symbol: &str, avg_price: Value);
}