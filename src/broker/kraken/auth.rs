
use base64::Engine;
use hmac::{Hmac, Mac};
use hyper::header::{ACCEPT, CONTENT_TYPE};
use reqwest;
use serde::Deserialize;
use sha2::{Digest, Sha256, Sha512};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const AUTH_TOKEN_BASE_URL: &str = "https://api.kraken.com";
const AUTH_TOKEN_PATH: &str = "/0/private/GetWebSocketsToken";

type HmacSha512 = Hmac<Sha512>;

#[derive(Deserialize)]
struct TokenResponse {
    error: Option<Vec<String>>,
    result: Option<TokenResult>,
}

#[derive(Deserialize)]
struct TokenResult {
    token: String,
    expires: u64,
}

#[derive(Default)]
struct AuthState {
    api_sign: Option<String>,
    token: Option<String>,
    expiration: Option<u64>,
}

pub struct Auth {
    api_key: String,
    api_secret: String,
    state: Arc<Mutex<AuthState>>,
    refresh_stop_tx: Option<mpsc::Sender<()>>,
    refresh_task: Option<JoinHandle<()>>,
}

impl Auth {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        Ok(Self {
            api_key: dotenv::var("KRAKEN_API_KEY")?,
            api_secret: dotenv::var("KRAKEN_API_SECRET")?,
            state: Arc::new(Mutex::new(AuthState::default())),
            refresh_stop_tx: None,
            refresh_task: None,
        })
    }

    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let api_key = self.api_key.clone();
        let api_secret = self.api_secret.clone();
        let state = Arc::clone(&self.state);

        let init_result = thread::spawn(move || {
            Auth::refresh_token(&api_key, &api_secret, &state)
                .map(|_| ())
                .map_err(|e| format!("{e:?}"))
        })
        .join()
        .map_err(|_| "Auth init worker thread panicked".to_string())?;

        if let Err(err) = init_result {
            return Err(err.into());
        }

        self.start_refresh_timer();
        Ok(())
    }

    fn get_nonce() -> u64 {
        let start = std::time::SystemTime::now();
        let since_epoch = start.duration_since(std::time::UNIX_EPOCH).expect("Expected system time but got an error");
        since_epoch.as_millis() as u64
    }

    pub fn get_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        self
            .state
            .lock()
            .unwrap()
            .token
            .clone()
            .ok_or_else(|| "Auth token not available".into())
    }

    fn sign_with_secret(api_secret: &str, path: &str, nonce: u64, post_data: &str) -> Result<String, Box<dyn std::error::Error>> {
        let secret = base64::engine::general_purpose::STANDARD.decode(api_secret)?;
        let mut message = path.as_bytes().to_vec();
        let nonce = nonce.to_string();
        let sha256_payload = Sha256::digest(format!("{}{}", nonce, post_data).as_bytes());
        message.extend_from_slice(&sha256_payload);

        let mut mac = HmacSha512::new_from_slice(&secret)?;
        mac.update(&message);

        Ok(base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes()))
    }

    fn refresh_token(api_key: &str, api_secret: &str, state: &Arc<Mutex<AuthState>>) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .build()?;
        let url = format!("{}{}", AUTH_TOKEN_BASE_URL, AUTH_TOKEN_PATH);
        let nonce = Self::get_nonce();
        let body = serde_json::json!({
            "nonce": nonce,
        })
        .to_string();
        let signature = Self::sign_with_secret(api_secret, AUTH_TOKEN_PATH, nonce, &body)?;

        let res = client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header("API-Key", api_key)
            .header("API-Sign", &signature)
            .body(body)
            .send()
            .map_err(|e| std::io::Error::other(format!("Kraken token request transport error at {url}: {e}")))?;

        let response_text = res.text()?;
        let response: TokenResponse = serde_json::from_str(&response_text)?;
        if let Some(errors) = response.error {
            if !errors.is_empty() {
                return Err(std::io::Error::other(format!("Kraken token request rejected: {}", errors.join(", "))).into());
            }
        }

        let result = response
            .result
            .ok_or_else(|| std::io::Error::other("Kraken token response missing result payload"))?;
        let token = result.token.clone();

        let mut state = state.lock().unwrap();
        state.api_sign = Some(signature);
        state.expiration = Some(result.expires);
        state.token = Some(token.clone());

        Ok(token)
    }

    fn start_refresh_timer(&mut self) {
        if self.refresh_task.is_some() {
            return;
        }

        let (stop_tx, stop_rx) = mpsc::channel();
        self.refresh_stop_tx = Some(stop_tx);

        let api_key = self.api_key.clone();
        let api_secret = self.api_secret.clone();
        let state = Arc::clone(&self.state);

        self.refresh_task = Some(thread::spawn(move || loop {
            let wait_secs = {
                let guard = state.lock().unwrap();
                guard.expiration.unwrap_or(900).saturating_sub(30).max(30)
            };

            if stop_rx.recv_timeout(Duration::from_secs(wait_secs)).is_ok() {
                break;
            }

            if let Err(err) = Self::refresh_token(&api_key, &api_secret, &state) {
                eprintln!("Error refreshing Kraken auth token: {}", err);
            }
        }));
    }
}

impl Drop for Auth {
    fn drop(&mut self) {
        if let Some(stop_tx) = self.refresh_stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(task) = self.refresh_task.take() {
            let _ = task.join();
        }
    }
}