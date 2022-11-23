#!/usr/bin/env sh
set -eu
cargo build --release --target wasm32-unknown-unknown
# optimize with wasm-opt
wasm-opt -Oz -o target/wasm32-unknown-unknown/release/timely-defuse.opt.wasm \
    target/wasm32-unknown-unknown/release/timely-defuse.wasm
# replace
cp -f target/wasm32-unknown-unknown/release/timely-defuse.opt.wasm \
    target/wasm32-unknown-unknown/release/timely-defuse.wasm \

wasm-bindgen --out-name timely-defuse \
    --out-dir wasm/target \
    --target web target/wasm32-unknown-unknown/release/timely-defuse.wasm
cp -rn assets/* wasm/assets/
