RUST_BACKTRACE=full RUST_LOG=info nohup /root/release/examples/bft start -c /data/c1.toml 1> /tmp/c1.log 2>&1 &
RUST_BACKTRACE=full RUST_LOG=info nohup /root/release/examples/bft start -c /data/c2.toml 1> /tmp/c2.log 2>&1 &
RUST_BACKTRACE=full RUST_LOG=info nohup /root/release/examples/bft start -c /data/c3.toml 1> /tmp/c3.log 2>&1 &
RUST_BACKTRACE=full RUST_LOG=info nohup /root/release/examples/bft start -c /data/c4.toml 1> /tmp/c4.log 2>&1 &
RUST_BACKTRACE=full RUST_LOG=info nohup /root/release/examples/bft start -c /data/c5.toml 1> /tmp/c5.log 2>&1 &
