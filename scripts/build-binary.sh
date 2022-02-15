#!/usr/bin/env bash

set -xe

CHAIN=$CHAIN
ARCHIVE_PATH=$ARCHIVE_PATH

echo "Rust version: $(rustc --version)"

echo  "Current Path = $PWD"

echo "ARCHIVE_PATH=$ARCHIVE_PATH"
mkdir -p $ARCHIVE_PATH

BIN_OUTPUT_PATH=target/release

# Build binary
SKIP_WASM_BUILD= cargo build --release --features with-$CHAIN-runtime,evm-tracing

# Move built binary to archive path
cp $BIN_OUTPUT_PATH/bholdus $ARCHIVE_PATH/bholdus
