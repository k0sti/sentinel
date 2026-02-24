use nostr::prelude::*;
use crate::geohash_util;

/// Parsed location from a Nostr event.
#[derive(Debug, Clone)]
pub struct ParsedLocation {
    pub geohash: String,
    pub lat: f64,
    pub lon: f64,
    pub accuracy: Option<f64>,
    pub d_tag: String,
    pub timestamp: Timestamp,
    pub kind: u16,
    pub pubkey: String,
}

/// Parse a public location event (kind 30472).
pub fn parse_public_event(event: &Event) -> Result<ParsedLocation, Box<dyn std::error::Error>> {
    if event.kind != Kind::from(30472) {
        return Err("Not a kind 30472 event".into());
    }

    let geohash = find_tag_value(event, "g")
        .ok_or("Missing g (geohash) tag")?;

    let d_tag = find_tag_value(event, "d")
        .unwrap_or_default();

    let accuracy = find_tag_value(event, "accuracy")
        .and_then(|v| v.parse::<f64>().ok());

    let (lat, lon) = geohash_util::decode(&geohash)?;

    Ok(ParsedLocation {
        geohash,
        lat,
        lon,
        accuracy,
        d_tag,
        timestamp: event.created_at,
        kind: 30472,
        pubkey: event.pubkey.to_hex(),
    })
}

/// Parse an encrypted location event (kind 30473).
/// Requires the decrypted content (plaintext tag array JSON).
pub fn parse_encrypted_content(
    event: &Event,
    decrypted_content: &str,
) -> Result<ParsedLocation, Box<dyn std::error::Error>> {
    if event.kind != Kind::from(30473) {
        return Err("Not a kind 30473 event".into());
    }

    let tags: Vec<Vec<String>> = serde_json::from_str(decrypted_content)?;

    let geohash = tags.iter()
        .find(|t| t.first().map(|s| s == "g").unwrap_or(false))
        .and_then(|t| t.get(1))
        .ok_or("Missing g tag in decrypted content")?
        .clone();

    let accuracy = tags.iter()
        .find(|t| t.first().map(|s| s == "accuracy").unwrap_or(false))
        .and_then(|t| t.get(1))
        .and_then(|v| v.parse::<f64>().ok());

    let d_tag = find_tag_value(event, "d").unwrap_or_default();
    let (lat, lon) = geohash_util::decode(&geohash)?;

    Ok(ParsedLocation {
        geohash,
        lat,
        lon,
        accuracy,
        d_tag,
        timestamp: event.created_at,
        kind: 30473,
        pubkey: event.pubkey.to_hex(),
    })
}

fn find_tag_value(event: &Event, tag_name: &str) -> Option<String> {
    event.tags.iter().find_map(|t| {
        let s = t.as_slice();
        if s.first().map(|v| v == tag_name).unwrap_or(false) {
            s.get(1).cloned()
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_builder;
    use crate::config::TrackingConfig;

    #[test]
    fn parse_public_roundtrip() {
        let config = TrackingConfig::default();
        let keys = Keys::generate();

        let builder = event_builder::build_public_event(60.17, 24.94, Some(10.0), &config).unwrap();
        let event = event_builder::sign_event(builder, &keys).unwrap();

        let parsed = parse_public_event(&event).unwrap();
        assert!((parsed.lat - 60.17).abs() < 0.001);
        assert!((parsed.lon - 24.94).abs() < 0.001);
        assert_eq!(parsed.d_tag, "default");
        assert_eq!(parsed.kind, 30472);
        assert!(parsed.accuracy.is_some());
    }

    #[test]
    fn parse_encrypted_roundtrip() {
        let sender = Keys::generate();
        let receiver = Keys::generate();
        let config = TrackingConfig::default();

        let payload = event_builder::build_encrypted_payload(60.17, 24.94, Some(5.0), 8).unwrap();

        let encrypted = nip44::encrypt(
            sender.secret_key(),
            &receiver.public_key(),
            &payload,
            nip44::Version::V2,
        ).unwrap();

        let builder = event_builder::build_encrypted_event(
            &encrypted,
            &receiver.public_key().to_hex(),
            &config,
        ).unwrap();
        let event = event_builder::sign_event(builder, &sender).unwrap();

        let decrypted = nip44::decrypt(
            receiver.secret_key(),
            &sender.public_key(),
            &event.content,
        ).unwrap();

        let parsed = parse_encrypted_content(&event, &decrypted).unwrap();
        assert!((parsed.lat - 60.17).abs() < 0.001);
        assert_eq!(parsed.kind, 30473);
    }
}
