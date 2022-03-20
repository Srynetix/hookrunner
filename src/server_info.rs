use serde::Serialize;

pub const APP_NAME: &str = "hookrunner";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
pub struct ServerInfo {
    message: String,
    version: String,
}

impl ServerInfo {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            message: format!("{}, ready for action!", APP_NAME),
            version: APP_VERSION.into(),
        }
    }
}
