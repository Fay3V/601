#!/usr/bin/bash

cargo run --release --features=headers --bin generate-headers
cp sm.h ../lab

cargo zigbuild --target x86_64-unknown-linux-gnu.2.19 --release --lib
cp target/x86_64-unknown-linux-gnu/release/libsm.so ../lab

