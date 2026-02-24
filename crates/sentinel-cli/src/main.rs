use clap::{Parser, Subcommand};
use nostr_sdk::prelude::*;
use sentinel_core::event_parser;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "sentinel", about = "Query Nostr location events")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Query latest location events for a pubkey
    Query {
        /// Public key (hex or npub)
        #[arg(long)]
        pubkey: String,

        /// Relay URLs
        #[arg(long, default_value = "wss://zooid.atlantislabs.space")]
        relays: Vec<String>,

        /// Filter by d-tag
        #[arg(long)]
        d_tag: Option<String>,

        /// nsec to decrypt kind 30473 events
        #[arg(long)]
        decrypt_with: Option<String>,
    },

    /// Follow a pubkey and alert on missing updates
    Follow {
        /// Public key (hex or npub)
        #[arg(long)]
        pubkey: String,

        /// Alert if no update within this duration (e.g. "5m", "1h")
        #[arg(long)]
        alert_after: String,

        /// Webhook URL to POST alerts to
        #[arg(long)]
        webhook: Option<String>,

        /// Relay URLs
        #[arg(long, default_value = "wss://zooid.atlantislabs.space")]
        relays: Vec<String>,
    },

    /// Show configured identity
    Whoami,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Query {
            pubkey,
            relays,
            d_tag,
            decrypt_with,
        } => {
            cmd_query(&pubkey, &relays, d_tag.as_deref(), decrypt_with.as_deref()).await?;
        }
        Commands::Follow {
            pubkey,
            alert_after,
            webhook,
            relays,
        } => {
            cmd_follow(&pubkey, &alert_after, webhook.as_deref(), &relays).await?;
        }
        Commands::Whoami => {
            eprintln!("No identity configured (CLI uses --decrypt-with for decryption)");
        }
    }

    Ok(())
}

fn parse_pubkey(input: &str) -> Result<PublicKey> {
    if input.starts_with("npub") {
        Ok(PublicKey::from_bech32(input)?)
    } else {
        Ok(PublicKey::from_hex(input)?)
    }
}

fn parse_duration_str(s: &str) -> std::result::Result<Duration, String> {
    let s = s.trim();
    let (num_str, multiplier) = if s.ends_with('h') {
        (&s[..s.len() - 1], 3600u64)
    } else if s.ends_with('m') {
        (&s[..s.len() - 1], 60u64)
    } else if s.ends_with('s') {
        (&s[..s.len() - 1], 1u64)
    } else {
        (s, 1u64)
    };

    let num: u64 = num_str.parse().map_err(|_| format!("Invalid duration: {}", s))?;
    Ok(Duration::from_secs(num * multiplier))
}

async fn cmd_query(
    pubkey_str: &str,
    relays: &[String],
    d_tag: Option<&str>,
    decrypt_with: Option<&str>,
) -> Result<()> {
    let pubkey = parse_pubkey(pubkey_str)?;
    let client = Client::default();

    for relay in relays {
        client.add_relay(relay).await?;
    }
    client.connect().await;

    let mut filter = Filter::new()
        .author(pubkey)
        .kinds(vec![Kind::from(30472), Kind::from(30473)])
        .limit(20);

    if let Some(d) = d_tag {
        filter = filter.identifier(d);
    }

    let events = client
        .fetch_events(vec![filter], Some(Duration::from_secs(10)))
        .await?;

    let decrypt_keys = match decrypt_with {
        Some(nsec) => {
            let keys = if nsec.starts_with("nsec") {
                Keys::parse(nsec)?
            } else {
                Keys::new(SecretKey::from_hex(nsec)?)
            };
            Some(keys)
        }
        None => None,
    };

    for event in events.iter() {
        match event.kind.as_u16() {
            30472 => {
                if let Ok(loc) = event_parser::parse_public_event(event) {
                    println!(
                        "[{}] kind:{} d:{} geohash:{} lat:{:.6} lon:{:.6} acc:{:?}",
                        loc.timestamp.to_human_datetime(),
                        loc.kind,
                        loc.d_tag,
                        loc.geohash,
                        loc.lat,
                        loc.lon,
                        loc.accuracy,
                    );
                }
            }
            30473 => {
                if let Some(ref keys) = decrypt_keys {
                    match nip44::decrypt(
                        keys.secret_key(),
                        &event.pubkey,
                        &event.content,
                    ) {
                        Ok(decrypted) => {
                            if let Ok(loc) =
                                event_parser::parse_encrypted_content(event, &decrypted)
                            {
                                println!(
                                    "[{}] kind:{} d:{} geohash:{} lat:{:.6} lon:{:.6} acc:{:?} (decrypted)",
                                    loc.timestamp.to_human_datetime(),
                                    loc.kind,
                                    loc.d_tag,
                                    loc.geohash,
                                    loc.lat,
                                    loc.lon,
                                    loc.accuracy,
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to decrypt event {}: {}", event.id, e);
                        }
                    }
                } else {
                    println!(
                        "[{}] kind:30473 (encrypted, use --decrypt-with to decode)",
                        event.created_at.to_human_datetime(),
                    );
                }
            }
            _ => {}
        }
    }

    client.disconnect().await;
    Ok(())
}

async fn cmd_follow(
    pubkey_str: &str,
    alert_after_str: &str,
    webhook: Option<&str>,
    relays: &[String],
) -> Result<()> {
    let pubkey = parse_pubkey(pubkey_str)?;
    let alert_duration = parse_duration_str(alert_after_str)
        .expect("Invalid duration format (use e.g. 5m, 1h, 30s)");

    let client = Client::default();
    for relay in relays {
        client.add_relay(relay).await?;
    }
    client.connect().await;

    let filter = Filter::new()
        .author(pubkey)
        .kinds(vec![Kind::from(30472), Kind::from(30473)])
        .since(Timestamp::now());

    client.subscribe(vec![filter], None).await?;

    eprintln!(
        "Following {} — alert after {:?} of silence",
        pubkey.to_bech32()?,
        alert_duration,
    );

    let last_event_time = Arc::new(Mutex::new(std::time::Instant::now()));
    let last_event_clone = Arc::clone(&last_event_time);
    let webhook_url = webhook.map(|s| s.to_string());
    let alert_dur = alert_duration;

    // Spawn alert checker
    let pubkey_bech32 = pubkey.to_bech32()?;
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            let elapsed = last_event_clone.lock().unwrap().elapsed();
            if elapsed >= alert_dur {
                let msg = format!(
                    "ALERT: No location update from {} for {:?}",
                    pubkey_bech32, alert_dur,
                );
                eprintln!("{}", msg);

                if let Some(ref url) = webhook_url {
                    let body = serde_json::json!({ "text": msg });
                    let _ = reqwest::Client::new()
                        .post(url)
                        .json(&body)
                        .send()
                        .await;
                }

                // Reset timer
                *last_event_clone.lock().unwrap() = std::time::Instant::now();
            }
        }
    });

    // Process notifications
    client
        .handle_notifications(|notification| {
            let last_event_time = Arc::clone(&last_event_time);
            async move {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    let kind = event.kind.as_u16();
                    if kind == 30472 || kind == 30473 {
                        eprintln!("Location update received (kind {})", kind);
                        *last_event_time.lock().unwrap() = std::time::Instant::now();
                    }
                }
                Ok(false) // false = don't stop
            }
        })
        .await?;

    client.disconnect().await;
    Ok(())
}
