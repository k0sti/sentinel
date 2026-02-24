use serde::{Deserialize, Serialize};

/// Configuration for location tracking and event publishing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    /// Interval between location publishes, in seconds.
    pub interval_secs: u64,
    /// Geohash precision (1-12 characters).
    pub precision: u8,
    /// Whether to encrypt location (kind 30473 vs 30472).
    pub encrypted: bool,
    /// Recipient pubkeys for encrypted events (hex).
    pub recipient_pubkeys: Vec<String>,
    /// Relay URLs to publish to.
    pub relays: Vec<String>,
    /// The `d` tag identifier (e.g. "phone", "car").
    pub d_tag: String,
    /// Expiration TTL in seconds (added to current time).
    pub expiration_secs: u64,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            interval_secs: 60,
            precision: 8,
            encrypted: false,
            recipient_pubkeys: Vec::new(),
            relays: vec!["wss://zooid.atlantislabs.space".to_string()],
            d_tag: "default".to_string(),
            expiration_secs: 3600,
        }
    }
}
