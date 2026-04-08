use crate::indicators::bollinger_bands::BollingerBands;
use crate::domain::market::MarketEvent;
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
    fn on_event(&mut self, event: &MarketEvent) {
        if let MarketEvent::KLine(kline) = event {
            let close_price = kline.close;
            let exchange = kline.exchange;

            if let Some((upper_band, _middle_band, lower_band)) = self.bollinger_bands.update(close_price) {
                if close_price < lower_band && !self.in_position {
                    self.in_position = true;
                    println!("Buy signal for {} at price {} on exchange {}", kline.symbol, close_price, exchange);
                } else if close_price > upper_band && self.in_position {
                    self.in_position = false;
                    println!("Sell signal for {} at price {} on exchange {}", kline.symbol, close_price, exchange);
                }
            }
        }
    }
}