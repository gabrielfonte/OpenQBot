use crate::broker::account::OrderFillType;
use serde::Serialize;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use crate::broker::binance::request;
use crate::broker::account::{Account, BalanceType, OrderSide};

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct MarketOrderParams {
    pub symbol: String,
    pub side: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "quantity", skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    #[serde(rename = "quoteOrderQty", skip_serializing_if = "Option::is_none")]
    pub quote_order_qty: Option<f64>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct LimitOrderParams {
    pub symbol: String,
    pub side: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub quantity: f64,
    pub price: f64,
    #[serde(rename = "timeInForce")]
    pub time_in_force: String,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct StopLimitOrderParams {
    pub symbol: String,
    pub side: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub quantity: f64,
    pub price: f64,
    #[serde(rename = "timeInForce")]
    pub time_in_force: String,
    #[serde(rename = "stopPrice", skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(rename = "trailingDelta", skip_serializing_if = "Option::is_none")]
    pub trailing_delta: Option<f64>,
}

#[derive(Clone)]
pub struct BinanceAccount {
    api_key: String,
    secret_key: String,
}

impl Account for BinanceAccount {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        println!("API_KEY: {}", dotenv::var("BINANCE_API_KEY").unwrap_or("Not set".to_string()));
        println!("SECRET_KEY: {}", match dotenv::var("BINANCE_API_SECRET") {
            Ok(_key) => "Set".to_string(),
            Err(_err) => "Not set".to_string(),
        });
        Ok(Self {
            api_key:    dotenv::var("BINANCE_API_KEY")?,
            secret_key: dotenv::var("BINANCE_API_SECRET")?,
        })
    }

    async fn get_account_balance(&self, _balance_type: BalanceType, currency: &str) -> Result<f64, Box<dyn std::error::Error>> {

        let req = request::Request::new_signed(
            "account.status".to_string(),
            None,
            self.api_key.clone(),
            self.secret_key.clone(),
        );

        if let Some(free_balance) = req.send().await?["result"]["balances"]
            .as_array()
            .and_then(|balances| balances.iter().find(|a| a["asset"] == currency))
            .and_then(|asset| asset["free"].as_str())
            .and_then(|s| s.parse::<f64>().ok())
        {
            println!("{}", free_balance);
            Ok(free_balance)
        } else {
            Err("Balance not found".into())
        }
    }

    #[warn(unused_variables)] // TODO: do we need time limit?
    async fn place_market_order(&self, symbol: &str, side: OrderSide, quantity: Option<f64>, value: Option<f64>, _time_limit: Option<u64>) -> Result<Value, Box<dyn std::error::Error>> {
        if quantity.is_none() && value.is_none() {
            return Err("Must provide either quantity or value".into());
        }
        if quantity.is_some() && value.is_some() {
            return Err("Provide quantity or value, not both".into());
        }

        let order = MarketOrderParams {
            symbol:          symbol.to_uppercase(),
            side:            side.to_string(),
            r#type:          "MARKET".to_string(),
            quantity,
            quote_order_qty: value,
        };

        let req = request::Request::new_signed(
            "order.place".to_string(),
            Some(serde_json::to_value(order)?),
            self.api_key.clone(),
            self.secret_key.clone(),
        );

        req.send().await
    }

    #[warn(unused_variables)] // TODO: do we need time limit?
    async fn place_limit_order(&self, symbol: &str, side: OrderSide, quantity: f64, price: f64, fill_type: Option<OrderFillType>, _time_limit: Option<u64>) -> Result<Value, Box<dyn std::error::Error>> {
        let order = LimitOrderParams {
            symbol:          symbol.to_uppercase(),
            side:            side.to_string(),
            r#type:          "LIMIT".to_string(),
            quantity,
            price,
            time_in_force:   fill_type.unwrap_or(OrderFillType::GoodTilCanceled).to_string(),
        };

        let req = request::Request::new_signed(
            "order.place".to_string(),
            Some(serde_json::to_value(order)?),
            self.api_key.clone(),
            self.secret_key.clone(),
        );

        req.send().await
    }

    #[warn(unused_variables)] // TODO: do we need time limit?
    async fn place_stop_limit_order(&self, symbol: &str, side: OrderSide, quantity: f64, limit_price: f64, stop_price: Option<f64>, trailing_delta: Option<f64>, fill_type: Option<OrderFillType>, _time_limit: Option<u64>) -> Result<Value, Box<dyn std::error::Error>> {
        if stop_price.is_none() && trailing_delta.is_none() {
            return Err("Must provide either stop price or trailing delta".into());
        }
        if stop_price.is_some() && trailing_delta.is_some() {
            return Err("Provide stop price or trailing delta, not both".into());
        }

        let order = StopLimitOrderParams {
            symbol:          symbol.to_uppercase(),
            side:            side.to_string(),
            r#type:          "STOP_LOSS_LIMIT".to_string(),
            quantity,
            price:           limit_price,
            time_in_force:   fill_type.unwrap_or(OrderFillType::GoodTilCanceled).to_string(),
            stop_price,
            trailing_delta,
        };

        let req = request::Request::new_signed(
            "order.place".to_string(),
            Some(serde_json::to_value(order)?),
            self.api_key.clone(),
            self.secret_key.clone(),
        );

        req.send().await
    }
    async fn cancel_order(
        &self,
        symbol: &str,
        order_id: u64,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let params = json!({
        "symbol": symbol.to_uppercase(),
        "orderId": order_id,
    });
        let req = request::Request::new_signed(
            "order.cancel".to_string(),
            Some(params),
            self.api_key.clone(),
            self.secret_key.clone(),
        );

        req.send().await
    }
}