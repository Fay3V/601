#!/usr/bin/bash

cargo run --release --features=headers --no-default-features --bin generate-headers
cp sm.h ../lab

cargo zigbuild  --no-default-features  --target x86_64-unknown-linux-gnu.2.19 --release --lib
cp target/x86_64-unknown-linux-gnu/release/libsm.so ../lab

