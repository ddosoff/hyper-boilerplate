use serde_derive::Deserialize;
use std::net::SocketAddr;
use toml;

const CONFIG_FILE_NAME: &str = "settings.toml";

// Config file structures
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub https: Option<HttpsServerSettings>,
    pub http: Option<HttpServerSettings>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpsServerSettings {
    pub host_port: SocketAddr,
    pub cert_pem: String,
    pub key_rsa_pem: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpServerSettings {
    pub host_port: SocketAddr,
}

impl Settings {
    pub fn read() -> Self {
        let settings_str = std::fs::read_to_string(CONFIG_FILE_NAME)
            .expect(&format!("Can't read {}", CONFIG_FILE_NAME));

        // Decode toml file to Settings struct using serde deserialization
        let settings: Settings =
            toml::from_str(&settings_str).expect(&format!("Can't decode {}", CONFIG_FILE_NAME));

        return settings;
    }
}
