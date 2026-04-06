use std::fmt;
use serde_json::Value;

#[derive(Eq, PartialEq, Hash, Clone)]
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

#[allow(dead_code)]
pub enum BalanceType {
    Spot,
    Margin,
    Futures,
    Financial,
}

impl fmt::Display for BalanceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BalanceType::Spot => write!(f, "SPOT"),
            BalanceType::Margin => write!(f, "MARGIN"),
            BalanceType::Futures => write!(f, "FUTURES"),
            BalanceType::Financial => write!(f, "FINANCIAL"),
        }
    }
}

#[allow(dead_code)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderSide::Buy  => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

#[allow(dead_code)]
pub enum OrderFillType {
    FillOrKill,
    ImmediateOrCancel,
    GoodTilCanceled,
}

impl fmt::Display for OrderFillType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderFillType::GoodTilCanceled => write!(f, "GTC"),
            OrderFillType::FillOrKill => write!(f, "FOK"),
            OrderFillType::ImmediateOrCancel => write!(f, "IOC"),
        }
    }
}
#[allow(dead_code)]
pub trait Account {
    fn new() -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;

    async fn get_account_balance(&self,
                                 balance_type: BalanceType,
                                 currency: &str
    ) -> Result<f64, Box<dyn std::error::Error>>;
    async fn place_market_order(&self,
                                symbol: &str,
                                side: OrderSide,
                                quantity: Option<f64>,
                                value: Option<f64>,
                                time_limit: Option<u64>
    ) -> Result<Value, Box<dyn std::error::Error>>;

    async fn place_limit_order(&self,
                               symbol: &str,
                               side: OrderSide,
                               quantity: f64,
                               price: f64,
                               fill_type: Option<OrderFillType>,
                               time_limit: Option<u64>
    ) -> Result<Value, Box<dyn std::error::Error>>;

    async fn place_stop_limit_order(&self,
                                    symbol: &str,
                                    side: OrderSide,
                                    quantity: f64,
                                    limit_price: f64,
                                    stop_price: Option<f64>,
                                    trailing_delta: Option<f64>,
                                    fill_type: Option<OrderFillType>,
                                    time_limit: Option<u64>) -> Result<Value, Box<dyn std::error::Error>>;

    async fn cancel_order(&self,
                          symbol: &str,
                          order_id: u64,
    ) -> Result<Value, Box<dyn std::error::Error>>;
}

