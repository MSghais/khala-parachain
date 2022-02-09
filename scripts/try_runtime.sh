#!/bin/bash

# SKIP_WASM_BUILD=1
cargo run --release --features try-runtime \
  -- try-runtime \
  -d /tmp/blackhole \
  --chain khala-staging-2004 \
  on-runtime-upgrade live \
  -u ws://127.0.0.1:9944 \
  --at 0xbb58b3fc28763ec13b5c4179249ebaec895efad1efa5b52b9402e6468b492787 \ # block-1174923
  --snapshot-path ./tmp/snapshot.bin |& tee ./tmp/sim.log

# -l trace,soketto=warn,jsonrpsee_ws_client=warn,remote-ext=warn,trie=warn,wasmtime_cranelift=warn \