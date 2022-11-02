use std::fmt;

#[derive(Debug, Clone)]
struct EnvError;

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "can't parse env to build EnvStore")
    }
}

fn get_env_var(var: &str) -> Result<String, EnvError> {
    std::env::var(var).map_err(|e| EnvError)
}

pub struct EnvStore {
    ws_url: String,
}

impl EnvStore {
    fn new(ws_url_var: &str) -> Result<EnvStore, EnvError> {
        Ok(EnvStore {
            ws_url: get_env_var(ws_url_var)?,
        })
    }

    fn get_ws_url(&self) -> &str {
        &self.ws_url
    }
}
