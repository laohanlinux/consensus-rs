# consensus-rs [![Build Status](https://travis-ci.org/laohanlinux/consensus-rs.svg?branch=master)](https://travis-ci.org/laohanlinux/consensus-rs)
Implement multiple blockchain consensus, including raft, pbft, paxos, dpos, power

- [x] pbft
- [ ] raft
- [ ] paxos
- [ ] dpos
- [ ] power

## start example

``` sh
# git submodule add --force https://github.com/laohanlinux/parity-common.git

# ./build.sh
```

## RUN Docker

``` sh
docker build -t tt .
```

## Debugging a Stuck Process

When the consensus node appears stuck (e.g. no new blocks), use these steps to find where it is blocked.

### 1. Enable trace logging

```sh
RUST_LOG=consensus=trace ./target/debug/consensus -c examples/c1.toml 2>&1 | tee /tmp/c1.log
```

Trace logs show seal wait cycles, commit events, and Core message handling. The last log line before the hang indicates where execution stopped.

### 2. Dump thread stack traces

When the process is stuck, from another terminal:

```sh
# Send SIGUSR2 to print debug hints
kill -USR2 $(pgrep -f "consensus.*c1")

# macOS: sample the process for 1 second to get stack traces of all threads
sample $(pgrep -f "consensus.*c1") 1

# Linux: use gdb to get backtraces
# gdb -p $(pgrep -f "consensus.*c1") -ex 'thread apply all bt' -ex quit
```

The `sample` output (macOS) or gdb output (Linux) shows each thread's call stack, so you can see which code path is blocking.
