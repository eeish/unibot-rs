use hex::FromHexError;
use std::fmt;

#[derive(Debug, Clone)]
struct EnvError {
    details: String,
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "can't parse env to build EnvStore, details: {}",
            self.details
        )
    }
}

impl From<()> for EnvError {
    fn from(err: ()) -> Self {
        EnvError {
            details: "default error for nothing".to_string(),
        }
    }
}

fn get_env_var(var: &str) -> Result<String, EnvError> {
    std::env::var(var).map_err(|e| EnvError {
        details: e.to_string(),
    })
}

// FIXME: handle error details
fn convert_val_to_key<const LEN: usize>(val: String) -> Result<[u8; LEN], EnvError> {
    Ok(hex::decode(&val).map_err(|e| {})?[..]
        .try_into()
        .map_err(|_| {})
        .unwrap())
}

pub struct EnvStore {
    ws_url: String,
    eth_private_key: [u8; 32],
}

impl EnvStore {
    fn new(ws_url_var: &str, eth_private_key_var: &str) -> Result<EnvStore, EnvError> {
        Ok(EnvStore {
            ws_url: get_env_var(ws_url_var)?,
            eth_private_key: convert_val_to_key(get_env_var(eth_private_key_var)?).unwrap(),
        })
    }

    pub fn get_ws_url(&self) -> &str {
        &self.ws_url
    }

    pub fn get_private_key(&self) -> &[u8] {
        &self.eth_private_key
    }
}
