use std::time::{SystemTime, UNIX_EPOCH};
use super::ws;
use fastwebsockets::{Frame, Payload};
use fastwebsockets::OpCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use sha2::Sha256;
use hmac::{Hmac, Mac};
use hex;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub params: Option<Value>,
}
type HmacSha256 = Hmac<Sha256>;

impl Request {
    pub fn new_signed(
        method: String,
        params: Option<Value>,
        api_key: String,
        priv_key: String,
    ) -> Self {
        let req_time: u128 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        let uuid = Uuid::new_v4();
        let mut params = params.unwrap_or(json!({}));

        params["timestamp"] = json!(req_time);
        params["apiKey"] = json!(api_key);

        let signature = Request::sign(&priv_key, params.clone());
        params["signature"] = json!(signature);

        Request {
            id: uuid.to_string(),
            method,
            params: Some(params),
        }
    }

    #[warn(unused)]
    pub fn new_unsigned(method: &str, params: Option<Value>) -> Self {
        let uuid = Uuid::new_v4();
        let req_time: u128 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let mut params = params.unwrap_or(json!({}));
        params["timestamp"] = json!(req_time);

        Request {
            id: uuid.to_string(),
            method: method.to_string(),
            params: Some(params),
        }
    }

    fn sign(priv_key: &str, params: Value) -> String {
        let mut params_vec: Vec<(&str, &serde_json::Value)> = params.as_object().unwrap()
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        params_vec.sort_by(|a, b| a.0.cmp(b.0));

        let msg = params_vec.iter()
            .map(|(key, value)| {
                let value_str = match value {
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                format!("{}={}", key, value_str)
            })
            .collect::<Vec<String>>()
            .join("&");

        println!("msg: {}", msg);

        let mut mac = HmacSha256::new_from_slice(priv_key.as_bytes()).unwrap();
        mac.update(msg.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        hex::encode(code_bytes)
    }

    pub async fn send(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let domain = "ws-api.binance.com";
        let mut ws = ws::connect(domain, "ws-api/v3").await?;

        let message = serde_json::to_string(&self)?;

        println!("Sending: {}", message);

        // Send the message to the server
        ws.write_frame(Frame::text(Payload::from(message.as_bytes()))).await?;

        loop {
            let msg = match ws.read_frame().await {
                Ok(msg) => msg,
                Err(e) => {
                    println!("Error: {}", e);
                    ws.write_frame(Frame::close_raw(vec![].into())).await?;
                    break;
                }
            };

            match msg.opcode {
                OpCode::Text => {
                    let payload =
                        String::from_utf8(msg.payload.to_vec()).expect("Invalid UTF-8 data");
                    let deserialized = serde_json::from_str::<serde_json::Value>(&payload).unwrap_or(serde_json::json!({}));
                    if deserialized["status"].as_i64().unwrap() == 200 {
                        println!("Compra executada");
                        let _msg = format!("Order executed {} units for {}$", deserialized["result"]["executedQty"], deserialized["result"]["price"]);
                        return Ok(deserialized);
                    } else {
                        println!("Order failed");
                        break;
                    }
                }
                OpCode::Close => {
                    break;
                }
                _ => {}
        }

        }

        Err("Error".into())
    }
}