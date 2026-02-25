default:
    @just --list

# === Dependencies ===
install:
    cd packages/app && bun install

# === Web ===
dev: install
    cd packages/app && bun run dev

build-wasm:
    cd crates/sentinel-core && wasm-pack build --target web --out-dir ../../packages/wasm

build-web: build-wasm install
    cd packages/app && bun run build

# === Android ===
android: build-web
    cd packages/app && bunx cap sync android
    cd packages/app/android && ./gradlew assembleDebug
    @echo "✅ APK at packages/app/android/app/build/outputs/apk/debug/app-debug.apk"

android-sync: build-web
    cd packages/app && bunx cap sync android

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
    @echo "1. just android"
    @echo "2. just android-install"
    @echo "3. adb logcat -s Capacitor:V Sentinel:V"
    @echo "4. Manual: grant location permission, start tracking, walk around"
    @echo "5. Verify events on relay: just query-self"

query-self:
    cargo run -p sentinel-cli -- query --pubkey $(cat packages/app/.pubkey 2>/dev/null || echo "<your-npub>")

# === Clean ===
clean:
    rm -rf packages/wasm/sentinel_core*
    rm -rf packages/app/dist
    rm -rf packages/app/android/app/build
    cargo clean
