use serde_json::Value;
use crate::indicators::bollinger_bands::BollingerBands;
use super::strategy::TradingStrategy;

pub struct BollingerBounceTradingStrategy {
    bollinger_bands: BollingerBands,
    in_position: bool,
}

impl BollingerBounceTradingStrategy {
    pub fn new(period: usize, std_dev: f64) -> Self {
        Self {
            bollinger_bands: BollingerBands::new(period, std_dev),
            in_position: false,
        }
    }
}

impl TradingStrategy for BollingerBounceTradingStrategy {
    fn on_kline(&mut self, symbol: &str, interval: &str, kline: Value) {
        let close_price = kline["k"]["c"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        if let Some((upper_band, middle_band, lower_band)) = self.bollinger_bands.update(close_price) {
            if close_price < lower_band && !self.in_position {
                self.in_position = true;
                println!("Buy signal for {} at price {}", symbol, close_price);
            } else if close_price > upper_band && self.in_position {
                self.in_position = false;
                println!("Sell signal for {} at price {}", symbol, close_price);
            }
        }
    }

    fn on_trade(&mut self, symbol: &str, trade: Value) {
    }

    fn on_aggregate_trade(&mut self, symbol: &str, agg_trade: Value) {
    }

    fn on_average_price(&mut self, symbol: &str, avg_price: Value) {
    }
}