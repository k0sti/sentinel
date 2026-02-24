# Sentinel — Multiplatform Location Tracker on Nostr

## Overview

Sentinel is a privacy-focused background location tracking app that publishes location updates as addressable Nostr events (kinds 30472/30473). It runs on Android (Capacitor) and web, with a Rust core compiled to WASM, and a CLI tool for querying/alerting.

## Architecture

```
sentinel/
├── crates/
│   ├── sentinel-core/        # Rust: crypto, geohash, event building, config
│   └── sentinel-cli/         # Rust: query location events, alert on missing updates
├── packages/
│   ├── wasm/                  # wasm-pack output from sentinel-core
│   └── app/                   # TypeScript app (Vite + React + Capacitor)
│       ├── src/
│       ├── android/           # Capacitor Android project
│       ├── capacitor.config.ts
│       ├── vite.config.ts
│       └── package.json
├── spec/                      # This spec
├── flake.nix                  # Dev shell (Rust, Node, JDK, Android SDK)
├── justfile                   # Build commands
├── Cargo.toml                 # Workspace
└── README.md
```

## Nostr Location Events

Per [nostr-location spec](https://github.com/k0sti/nostr-location/blob/main/doc/NostrLocation.md):

### Kind 30472 — Public Location
```json
{
  "kind": 30472,
  "tags": [
    ["g", "<geohash>"],
    ["d", "<configurable-identifier>"],
    ["accuracy", "<meters>"],
    ["expiration", "<unix-ts>"]
  ]
}
```

### Kind 30473 — Encrypted Location
```json
{
  "kind": 30473,
  "tags": [
    ["p", "<recipient-pubkey>"],
    ["d", "<configurable-identifier>"],
    ["expiration", "<unix-ts>"]
  ],
  "content": "<NIP-44 encrypted: [[\"g\",\"<geohash>\"],[\"accuracy\",\"<meters>\"]]>"
}
```

- Addressable (replaceable by d-tag) — no location history clutter
- `d` tag is user-configurable (e.g. "car", "phone", "hike-2026")
- Encryption uses NIP-44 to recipient pubkey

## Rust Crate: `sentinel-core`

**Purpose:** Platform-agnostic location event logic, compiled to native + WASM.

### Features
- Geohash encoding (lat/lon → geohash string at configurable precision)
- Nostr event construction (kind 30472, 30473)
- NIP-44 encryption/decryption of location tags
- Config types: `TrackingConfig { interval_secs, precision, encrypted, recipient_pubkeys, relays, d_tag, expiration_secs }`
- Event signing (takes secret key or delegates to external signer)
- Event parsing/validation (decode location from events)

### WASM API
Exposed via `wasm-bindgen`:
- `build_location_event(lat, lon, accuracy, config) → SignedEvent | UnsignedEvent`
- `parse_location_event(event_json) → LocationData`
- `encode_geohash(lat, lon, precision) → string`

### Dependencies
- `nostr` crate (rust-nostr) for event types, NIP-44
- `geohash` crate
- `serde`, `wasm-bindgen`, `tsify`

## Relay Auth (NIP-42)

Sentinel must support authenticated relays (NIP-42 AUTH challenge-response). Default relay: `wss://zooid.atlantislabs.space`.

### Implementation
- **Rust (CLI):** `nostr-sdk` handles NIP-42 automatically when identity is configured
- **TypeScript (app):** `applesauce-relay` supports AUTH — pass signer to relay pool, it responds to AUTH challenges using the configured identity (nsec/NIP-07/Amber/NIP-46)
- **Config default:** relays list pre-populated with `wss://zooid.atlantislabs.space`

## Rust Crate: `sentinel-cli`

**Purpose:** Query location events from relays, alert on missing updates.

### Commands
```
sentinel query --pubkey <hex|npub> [--relays wss://...] [--d-tag <id>] [--decrypt-with <nsec>]
sentinel follow --pubkey <hex|npub> --alert-after <duration> [--webhook <url>] [--relays wss://...]
sentinel whoami  # show configured identity
```

### `query`
- Connects to relays, fetches latest 30472/30473 events for pubkey
- If `--decrypt-with` provided, decrypts kind 30473 content
- Outputs: timestamp, geohash, lat/lon, accuracy, d-tag

### `follow`
- Subscribes to location events from pubkey
- If no event received within `--alert-after` duration, triggers alert
- Alert options: stderr message, webhook POST, desktop notification
- Runs continuously until killed

### Dependencies
- `nostr-sdk` for relay connections
- `clap` for CLI
- `tokio` runtime

## TypeScript App: `packages/app`

### Stack
- **Framework:** React 18 + TypeScript
- **Build:** Vite
- **UI:** Konsta UI (mobile-first, iOS/Android native feel — better Capacitor support than shadcn)
- **Map:** Leaflet + react-leaflet
- **Nostr:** applesauce-core, applesauce-signers, applesauce-react, applesauce-relay, nostr-tools
- **Native:** Capacitor 7 (@capacitor/geolocation, @capawesome/background-runner)
- **WASM:** sentinel-core via wasm-pack

### Screens

1. **Home / Map**
   - Map showing own last published location
   - Start/Stop tracking toggle (prominent button)
   - Status: tracking active/paused, last update time, relay connection status
   - Quick settings: interval, precision

2. **Settings**
   - **Identity:** nsec input, Nostr Connect (NIP-46), Amber signer (Android), web extension (nos2x/Alby)
   - **Share config:** encrypted vs public, recipient pubkeys, d-tag identifier
   - **Relays:** configurable relay list
   - **Tracking:** interval (seconds), geohash precision, expiration TTL
   - **Background:** enable/disable background tracking (Android)

3. **Following** (optional v2)
   - Subscribe to others' location events
   - Show on map

### Identity / Signing

Priority order per platform:

**Android:**
1. Amber (via intent — `nostrsigner:` URI scheme)
2. nsec (stored in Capacitor Preferences, encrypted at rest)
3. Nostr Connect (NIP-46)

**Web:**
1. NIP-07 browser extension (nos2x, Alby)
2. nsec (localStorage, warn user)
3. Nostr Connect (NIP-46)

Use `applesauce-signers` which supports NIP-07, NIP-46, and nsec. Amber integration needs a thin wrapper using Capacitor App plugin for intent handling.

### Background Location (Android)

- Use `@capacitor/geolocation` for foreground
- Use `@capawesome/capacitor-background-runner` for background task execution
- Background task: wake periodically, get location, call WASM to build event, publish to relays
- Android foreground service notification: "Sentinel is tracking your location"
- Configurable: background on/off, interval

### State Management

- `applesauce-core` observables (RxJS) for Nostr state
- React context for tracking state, config
- Persist config in Capacitor Preferences (Android) / localStorage (web)

## Build System

### flake.nix

Dev shell providing:
- Rust nightly + wasm32-unknown-unknown target (via rust-overlay, like crossworld)
- wasm-pack
- Node.js 22, bun
- JDK 21
- android-tools (adb)
- just
- playwright (for web tests)

### justfile

```just
default:
    @just --list

# === Web ===
dev:
    cd packages/app && bun run dev

build-wasm:
    cd crates/sentinel-core && wasm-pack build --target web --out-dir ../../packages/wasm

build-web: build-wasm
    cd packages/app && bun run build

# === Android ===
android-sync: build-web
    cd packages/app && bunx cap sync android

android-build: build-web
    cd packages/app && bunx cap sync android && cd android && ./gradlew assembleDebug

android-run: android-sync
    cd packages/app && bunx cap run android

android-install:
    adb install packages/app/android/app/build/outputs/apk/debug/app-debug.apk

# === CLI ===
build-cli:
    cargo build --release -p sentinel-cli

# === Test ===
test-rust:
    cargo test

test-web:
    cd packages/app && bunx playwright test

test-android-usb:
    @echo "Prerequisites: USB-connected device with developer mode"
    @echo "1. just android-build"
    @echo "2. just android-install"
    @echo "3. adb logcat -s Capacitor:V Sentinel:V"
    @echo "4. Manual: grant location permission, start tracking, walk around"
    @echo "5. Verify events on relay: just query-self"

query-self:
    cargo run -p sentinel-cli -- query --pubkey $(cat packages/app/.pubkey 2>/dev/null || echo "<your-npub>")
```

## Testing Strategy

### Rust (unit)
- `sentinel-core`: geohash encoding, event building, encryption round-trip, config serde
- `sentinel-cli`: event parsing, alert timing logic
- Run: `cargo test`

### Web (Playwright)
- App renders, start/stop button works
- Settings form saves/loads config
- Mock geolocation API → verify event constructed correctly
- Identity: test nsec input flow, NIP-07 mock
- Run: `bunx playwright test`

### Android (USB device)
Manual + semi-automated:
1. `just android-build && just android-install`
2. Grant location permission when prompted
3. Start tracking, verify via:
   - `adb logcat -s Capacitor:V` — check location events being built
   - `sentinel-cli query --pubkey <your-pubkey>` — verify events on relay
4. Test background: minimize app, wait for interval, check relay for new events
5. Test stop: press stop, verify no new events published

### Integration
- Use a local/test relay (e.g. `strfry` or `nostream`) for CI
- Publish event from app/WASM → query with CLI → assert match

## Open Questions

- Konsta UI vs shadcn — Konsta gives native iOS/Android feel out of box, shadcn needs more custom work for mobile. Recommendation: Konsta for v1.
- Background tracking battery impact — need to tune interval, use significant-change location mode where possible
- Gift-wrap (NIP-59) for better metadata privacy? v2 consideration
- History mode — currently addressable (replaces), could optionally support non-addressable events for track recording

## v1 Scope

1. ✅ Rust core: geohash + event building + encryption
2. ✅ WASM bindings
3. ✅ App: map, start/stop, settings, identity (nsec + NIP-07 + Amber)
4. ✅ Background tracking (Android)
5. ✅ CLI: query + follow with alert
6. ✅ Flake + justfile
7. ✅ Playwright web tests
8. ❌ Following others on map (v2)
9. ❌ Track history mode (v2)
