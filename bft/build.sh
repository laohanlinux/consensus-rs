#!/bin/bash

cargo build --example bft

RUST_BACKTRACE=1 RUST_LOG=trace ./target/debug/examples/bft start --config examples/config.toml
