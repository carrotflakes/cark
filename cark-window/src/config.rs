pub fn load_config() -> Config {
    let path = "cark.toml";
    let Ok(config) = std::fs::read_to_string(path) else {
        return Config::default();
    };
    toml::from_str(&config).unwrap()
}

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub server_tcp_addr: String,
    pub server_udp_addr: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_tcp_addr: "127.0.0.1:8080".to_string(),
            server_udp_addr: "127.0.0.1:8081".to_string(),
        }
    }
}
