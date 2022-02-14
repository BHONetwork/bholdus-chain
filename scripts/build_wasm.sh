#!/usr/bin/env bash

set -xe

CHAIN=$CHAIN
ARCHIVE_PATH=$ARCHIVE_PATH

echo  "Current Path = $PWD"

echo "ARCHIVE_PATH=$ARCHIVE_PATH"
mkdir -p $ARCHIVE_PATH

SRTOOL_WASM_OUTPUT_PATH=runtime/$CHAIN/target/srtool/release/wbuild/$CHAIN-runtime
echo "SRTOOL_WASM_OUTPUT_PATH=$SRTOOL_WASM_OUTPUT_PATH"

# Build wasm using srtool cli
srtool build --build-opts \""--features evm-tracing"\" --package $CHAIN-runtime $PWD

# Move built wasm to archive path
mv $SRTOOL_WASM_OUTPUT_PATH/${CHAIN}_runtime.wasm $ARCHIVE_PATH/${CHAIN}_runtime.wasm
mv $SRTOOL_WASM_OUTPUT_PATH/${CHAIN}_runtime.compact.wasm $ARCHIVE_PATH/${CHAIN}_runtime.compact.wasm
mv $SRTOOL_WASM_OUTPUT_PATH/${CHAIN}_runtime.compact.compressed.wasm $ARCHIVE_PATH/${CHAIN}_runtime.compact.compressed.wasm

# Generate subwasm info for compact wasm
subwasm --json info $ARCHIVE_PATH/${CHAIN}_runtime.compact.wasm > $ARCHIVE_PATH/${CHAIN}-runtime-subwasm-info.json
subwasm info $ARCHIVE_PATH/${CHAIN}_runtime.compact.wasm > $ARCHIVE_PATH/${CHAIN}-runtime-subwasm-info.txt

# Generate subwasm info for compact compressed wasm
subwasm --json info $ARCHIVE_PATH/${CHAIN}_runtime.compact.compressed.wasm > $ARCHIVE_PATH/${CHAIN}-runtime-compressed-subwasm_info.json
subwasm info $ARCHIVE_PATH/${CHAIN}_runtime.compact.compressed.wasm > $ARCHIVE_PATH/$CHAIN-runtime-compressed-subwasm-info.txt
