use wasm_bindgen::prelude::*;
use crate::config::TrackingConfig;
use crate::event_builder;
use crate::event_parser;
use crate::geohash_util;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn encode_geohash(lat: f64, lon: f64, precision: u8) -> Result<String, JsError> {
    geohash_util::encode(lat, lon, precision).map_err(|e| JsError::new(&e.to_string()))
}

/// Build a public location event (kind 30472).
/// Returns unsigned event template JSON for the TS signer.
#[wasm_bindgen]
pub fn build_public_location_event(
    lat: f64,
    lon: f64,
    accuracy: f64,
    config_json: &str,
) -> Result<String, JsError> {
    let config: TrackingConfig =
        serde_json::from_str(config_json).map_err(|e| JsError::new(&e.to_string()))?;

    let acc = if accuracy >= 0.0 { Some(accuracy) } else { None };
    let builder = event_builder::build_public_event(lat, lon, acc, &config)
        .map_err(|e| JsError::new(&e.to_string()))?;

    // Use a dummy pubkey to get the unsigned event structure
    let dummy_pk = nostr::PublicKey::from_hex(
        "0000000000000000000000000000000000000000000000000000000000000001",
    ).unwrap();
    let unsigned = builder.build(dummy_pk);

    let result = serde_json::json!({
        "kind": 30472,
        "content": unsigned.content,
        "tags": unsigned.tags.iter().map(|t| t.as_slice().to_vec()).collect::<Vec<_>>(),
    });
    Ok(result.to_string())
}

/// Build the plaintext payload for NIP-44 encryption.
#[wasm_bindgen]
pub fn build_encrypted_payload(
    lat: f64,
    lon: f64,
    accuracy: f64,
    precision: u8,
) -> Result<String, JsError> {
    let acc = if accuracy >= 0.0 { Some(accuracy) } else { None };
    event_builder::build_encrypted_payload(lat, lon, acc, precision)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Build an encrypted location event (kind 30473).
/// Returns unsigned event template JSON.
#[wasm_bindgen]
pub fn build_encrypted_location_event(
    encrypted_content: &str,
    recipient_pubkey: &str,
    config_json: &str,
) -> Result<String, JsError> {
    let config: TrackingConfig =
        serde_json::from_str(config_json).map_err(|e| JsError::new(&e.to_string()))?;

    let builder = event_builder::build_encrypted_event(encrypted_content, recipient_pubkey, &config)
        .map_err(|e| JsError::new(&e.to_string()))?;

    let dummy_pk = nostr::PublicKey::from_hex(
        "0000000000000000000000000000000000000000000000000000000000000001",
    ).unwrap();
    let unsigned = builder.build(dummy_pk);

    let result = serde_json::json!({
        "kind": 30473,
        "content": unsigned.content,
        "tags": unsigned.tags.iter().map(|t| t.as_slice().to_vec()).collect::<Vec<_>>(),
    });
    Ok(result.to_string())
}

/// Parse a public location event JSON, returning location data JSON.
#[wasm_bindgen]
pub fn parse_location_event(event_json: &str) -> Result<String, JsError> {
    let event: nostr::Event =
        serde_json::from_str(event_json).map_err(|e| JsError::new(&e.to_string()))?;

    let parsed = event_parser::parse_public_event(&event)
        .map_err(|e| JsError::new(&e.to_string()))?;

    let result = serde_json::json!({
        "geohash": parsed.geohash,
        "lat": parsed.lat,
        "lon": parsed.lon,
        "accuracy": parsed.accuracy,
        "d_tag": parsed.d_tag,
        "timestamp": parsed.timestamp.as_u64(),
        "kind": parsed.kind,
        "pubkey": parsed.pubkey,
    });
    Ok(result.to_string())
}
