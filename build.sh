#!/bin/bash

version="nightly-2019-04-07"
rustup override set $version

pkill bft

cargo build --example bft

function cluster() {
    RUST_BACKTRACE=full RUST_LOG=info ./target/debug/examples/bft start --config examples/c1.toml 1> /tmp/c1.log 2>&1 &
    RUST_BACKTRACE=full RUST_LOG=info ./target/debug/examples/bft start --config examples/c2.toml 1> /tmp/c2.log 2>&1 &
    RUST_BACKTRACE=full RUST_LOG=info ./target/debug/examples/bft start --config examples/c3.toml 1> /tmp/c3.log 2>&1 &
    RUST_BACKTRACE=full RUST_LOG=info ./target/debug/examples/bft start --config examples/c4.toml 1> /tmp/c4.log 2>&1 &
    RUST_BACKTRACE=full RUST_LOG=info ./target/debug/examples/bft start --config examples/c5.toml 1> /tmp/c5.log 2>&1 &
}

echo "run in 5 nodes"
pkill bft
cluster
