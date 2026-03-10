use crate::indicators::sma::SMA;

pub struct BollingerBands {
    period: usize,
    multiplier: f64,
    sma: SMA,
    values: Vec<f64>,
}

impl BollingerBands {
    pub fn new(period: usize, multiplier: f64) -> Self {
        BollingerBands {
            period,
            multiplier,
            sma: SMA::new(period),
            values: Vec::new(),
        }
    }

    pub fn update(&mut self, value: f64) -> Option<(f64, f64, f64)> {
        self.values.push(value);
        if self.values.len() > self.period {
            self.values.remove(0);
        }
        if let Some(sma) = self.sma.update(value) {
            let std_dev = (self.values.iter().map(|v| (v - sma).powi(2)).sum::<f64>() / self.period as f64).sqrt();
            let upper_band = sma + self.multiplier * std_dev;
            let lower_band = sma - self.multiplier * std_dev;
            Some((upper_band, sma, lower_band))
        } else {
            None
        }
    }
}