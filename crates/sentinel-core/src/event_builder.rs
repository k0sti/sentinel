use nostr::prelude::*;
use crate::config::TrackingConfig;
use crate::geohash_util;

/// Location data extracted from or to be put into a Nostr event.
#[derive(Debug, Clone)]
pub struct LocationData {
    pub geohash: String,
    pub lat: f64,
    pub lon: f64,
    pub accuracy: Option<f64>,
    pub d_tag: String,
    pub encrypted: bool,
    pub timestamp: Option<Timestamp>,
}

/// Build a public location event (kind 30472).
pub fn build_public_event(
    lat: f64,
    lon: f64,
    accuracy: Option<f64>,
    config: &TrackingConfig,
) -> Result<EventBuilder, Box<dyn std::error::Error>> {
    let ghash = geohash_util::encode(lat, lon, config.precision)?;
    let expiration = Timestamp::from(Timestamp::now().as_u64() + config.expiration_secs);

    let mut tags = vec![
        Tag::custom(TagKind::custom("g"), vec![ghash]),
        Tag::identifier(&config.d_tag),
        Tag::expiration(expiration),
    ];
    if let Some(acc) = accuracy {
        tags.push(Tag::custom(TagKind::custom("accuracy"), vec![acc.to_string()]));
    }

    Ok(EventBuilder::new(Kind::from(30472), "").tags(tags))
}

/// Build an encrypted location event (kind 30473).
/// `encrypted_content` should be the NIP-44 ciphertext.
pub fn build_encrypted_event(
    encrypted_content: &str,
    recipient_pubkey: &str,
    config: &TrackingConfig,
) -> Result<EventBuilder, Box<dyn std::error::Error>> {
    let expiration = Timestamp::from(Timestamp::now().as_u64() + config.expiration_secs);
    let recipient = PublicKey::from_hex(recipient_pubkey)?;

    let tags = vec![
        Tag::public_key(recipient),
        Tag::identifier(&config.d_tag),
        Tag::expiration(expiration),
    ];

    Ok(EventBuilder::new(Kind::from(30473), encrypted_content).tags(tags))
}

/// Prepare the plaintext payload for NIP-44 encryption.
pub fn build_encrypted_payload(
    lat: f64,
    lon: f64,
    accuracy: Option<f64>,
    precision: u8,
) -> Result<String, Box<dyn std::error::Error>> {
    let ghash = geohash_util::encode(lat, lon, precision)?;
    let mut tags: Vec<Vec<String>> = vec![vec!["g".to_string(), ghash]];
    if let Some(acc) = accuracy {
        tags.push(vec!["accuracy".to_string(), acc.to_string()]);
    }
    Ok(serde_json::to_string(&tags)?)
}

/// Sign an event builder with the given secret key.
pub fn sign_event(
    builder: EventBuilder,
    keys: &Keys,
) -> Result<Event, Box<dyn std::error::Error>> {
    Ok(builder.sign_with_keys(keys)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keys() -> Keys {
        Keys::generate()
    }

    #[test]
    fn build_and_sign_public_event() {
        let config = TrackingConfig::default();
        let keys = test_keys();

        let builder = build_public_event(60.17, 24.94, Some(10.0), &config).unwrap();
        let event = sign_event(builder, &keys).unwrap();

        assert_eq!(event.kind, Kind::from(30472));

        let g_tag = event.tags.iter().find(|t| {
            t.as_slice().first().map(|s| s == "g").unwrap_or(false)
        });
        assert!(g_tag.is_some());

        let d_tag = event.tags.iter().find(|t| {
            t.as_slice().first().map(|s| s == "d").unwrap_or(false)
        });
        assert!(d_tag.is_some());
        assert_eq!(d_tag.unwrap().as_slice()[1], "default");
    }

    #[test]
    fn build_encrypted_payload_json() {
        let payload = build_encrypted_payload(60.17, 24.94, Some(5.0), 8).unwrap();
        let parsed: Vec<Vec<String>> = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed[0][0], "g");
        assert_eq!(parsed[0][1].len(), 8);
        assert_eq!(parsed[1][0], "accuracy");
        assert_eq!(parsed[1][1], "5");
    }

    #[test]
    fn build_encrypted_event_structure() {
        let config = TrackingConfig {
            encrypted: true,
            ..TrackingConfig::default()
        };
        let keys = test_keys();
        let recipient_keys = Keys::generate();
        let recipient_hex = recipient_keys.public_key().to_hex();

        let builder = build_encrypted_event(
            "encrypted-content-here",
            &recipient_hex,
            &config,
        ).unwrap();
        let event = sign_event(builder, &keys).unwrap();

        assert_eq!(event.kind, Kind::from(30473));
        assert_eq!(event.content, "encrypted-content-here");

        let p_tag = event.tags.iter().find(|t| {
            t.as_slice().first().map(|s| s == "p").unwrap_or(false)
        });
        assert!(p_tag.is_some());
    }

    #[test]
    fn nip44_encryption_roundtrip() {
        let sender = Keys::generate();
        let receiver = Keys::generate();

        let payload = build_encrypted_payload(60.17, 24.94, Some(10.0), 8).unwrap();

        let encrypted = nip44::encrypt(
            sender.secret_key(),
            &receiver.public_key(),
            &payload,
            nip44::Version::V2,
        ).unwrap();

        let decrypted = nip44::decrypt(
            receiver.secret_key(),
            &sender.public_key(),
            &encrypted,
        ).unwrap();

        assert_eq!(decrypted, payload);

        let tags: Vec<Vec<String>> = serde_json::from_str(&decrypted).unwrap();
        assert_eq!(tags[0][0], "g");
    }
}
