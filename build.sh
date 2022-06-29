#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
mkdir -p ./out
cp target/wasm32-unknown-unknown/release/ft.wasm out/ft.wasm
cp target/wasm32-unknown-unknown/release/nft.wasm out/nft.wasm
cp target/wasm32-unknown-unknown/release/marketplace.wasm out/marketplace.wasm
