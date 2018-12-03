#!/bin/bash

pkill bft

cargo build --example bft


function single() {
    RUST_BACKTRACE=1 RUST_LOG=debug ./target/debug/examples/bft start --config examples/config.toml
}

function cluster() {
    RUST_BACKTRACE=1 RUST_LOG=trace ./target/debug/examples/bft start --config examples/c1.toml &
    RUST_BACKTRACE=1 RUST_LOG=trace ./target/debug/examples/bft start --config examples/c2.toml &
    RUST_BACKTRACE=1 RUST_LOG=trace ./target/debug/examples/bft start --config examples/c3.toml &
    RUST_BACKTRACE=1 RUST_LOG=trace ./target/debug/examples/bft start --config examples/c4.toml &
    RUST_BACKTRACE=1 RUST_LOG=trace ./target/debug/examples/bft start --config examples/c5.toml &
}

case $1 in
    "single")
        echo "run in single modo"
        single
    ;;

    "cluster")
        echo "run in cluster modo"
        cluster
    ;;
esac