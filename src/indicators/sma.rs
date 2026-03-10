pub struct SMA {
    period: usize,
    values: Vec<f64>,
}

impl SMA {
    pub fn new(period: usize) -> Self {
        SMA {
            period,
            values: Vec::new(),
        }
    }

    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.values.push(value);
        if self.values.len() > self.period {
            self.values.remove(0);
        }
        if self.values.len() == self.period {
            Some(self.values.iter().sum::<f64>() / self.period as f64)
        } else {
            None
        }
    }
}