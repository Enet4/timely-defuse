#!/usr/bin/env sh
set -eu
cargo build --target wasm32-unknown-unknown
wasm-bindgen --out-name timely-defuse \
    --out-dir wasm/target \
    --target web target/wasm32-unknown-unknown/debug/timely-defuse.wasm
cp -rn assets/* wasm/assets/
