# Sentinel

Privacy-focused multiplatform location tracker on Nostr. Publishes location as addressable events (kinds 30472/30473) with optional NIP-44 encryption.

## Architecture

- **sentinel-core** — Rust crate: geohash encoding, Nostr event building (30472/30473), NIP-44 encryption, WASM bindings
- **sentinel-cli** — Rust CLI: query location events, follow pubkeys with alerting
- **packages/app** — React 18 + TypeScript + Vite + Konsta UI + Capacitor 7 + Leaflet
- **packages/wasm** — WASM output from sentinel-core

## Quick Start

```bash
# Enter dev shell
nix develop

# Run web app
just dev

# Build WASM + web
just build-web

# Run Rust tests
just test-rust

# Build CLI
just build-cli
```

## CLI Usage

```bash
# Query someone's public location
sentinel query --pubkey <npub|hex>

# Query and decrypt encrypted locations
sentinel query --pubkey <npub|hex> --decrypt-with <nsec>

# Follow with alerting
sentinel follow --pubkey <npub|hex> --alert-after 5m

# With webhook
sentinel follow --pubkey <npub|hex> --alert-after 1h --webhook https://hooks.example.com/alert
```

## Android

```bash
just android-build    # Build APK
just android-install  # Install via ADB
just android-run      # Run on connected device
```

## Nostr Events

### Kind 30472 — Public Location
Addressable event with geohash in `g` tag, configurable `d` tag identifier.

### Kind 30473 — Encrypted Location
NIP-44 encrypted content containing `[["g","<geohash>"],["accuracy","<meters>"]]`, with `p` tag for recipient.

## Default Relay

`wss://zooid.atlantislabs.space` (NIP-42 AUTH supported)

## Testing

```bash
just test-rust   # Rust unit tests
just test-web    # Playwright web tests
```
