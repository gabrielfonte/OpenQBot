#![allow(dead_code)]
use std::fmt;
use serde_json::Value;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
#[allow(dead_code)]
pub enum KlineInterval {
    OneSecond,
    OneMinute,
    ThreeMinutes,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    TwoHours,
    FourHours,
    SixHours,
    EightHours,
    TwelveHours,
    OneDay,
    ThreeDays,
    OneWeek,
    OneMonth,
}

impl fmt::Display for KlineInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            KlineInterval::OneSecond     => "1s",
            KlineInterval::OneMinute     => "1m",
            KlineInterval::ThreeMinutes  => "3m",
            KlineInterval::FiveMinutes   => "5m",
            KlineInterval::FifteenMinutes=> "15m",
            KlineInterval::ThirtyMinutes => "30m",
            KlineInterval::OneHour       => "1h",
            KlineInterval::TwoHours      => "2h",
            KlineInterval::FourHours     => "4h",
            KlineInterval::SixHours      => "6h",
            KlineInterval::EightHours    => "8h",
            KlineInterval::TwelveHours   => "12h",
            KlineInterval::OneDay        => "1d",
            KlineInterval::ThreeDays     => "3d",
            KlineInterval::OneWeek       => "1w",
            KlineInterval::OneMonth      => "1M",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub struct KlineEvent {
    pub exchange: &'static str,
    pub symbol: String,
    pub interval: KlineInterval,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
}

#[derive(Clone, Debug)]
pub struct TradeEvent {
    pub exchange: &'static str,
    pub symbol: String,
    pub price: Option<f64>,
    pub quantity: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct AggregateTradeEvent {
    pub exchange: &'static str,
    pub symbol: String,
    pub price: Option<f64>,
    pub quantity: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct AveragePriceEvent {
    pub exchange: &'static str,
    pub symbol: String,
    pub average_price: Option<f64>,
}

#[derive(Clone, Debug)]
pub enum MarketEvent {
    KLine(KlineEvent),
    Trade(TradeEvent),
    AggregateTrade(AggregateTradeEvent),
    AveragePrice(AveragePriceEvent),
    Unknown(Value),
}
