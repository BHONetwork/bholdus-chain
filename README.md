# BHO Chain Node

[![BHO Chain](https://img.shields.io/badge/Bholdus-brightgreen?logo=Parity%20Substrate)](https://apps.bholdus.com/)

A fresh FRAME-based [Substrate](https://www.substrate.io/) node

## Getting Started

BHO Blockchain Network is interoperable, cross-chain with various digital asset economies and Defi Networks such as Binance Smart Chain, Ethereum, and Polkadot.

## Install required packages

To install required packages on macOS or Linux:

- Ubuntu or Debian: ```sudo apt update && sudo apt install -y git clang curl libssl-dev llvm libudev-dev```
- Arch Linux:	```pacman -Syu --needed --noconfirm curl git clang```
- Fedora:	```sudo dnf update sudo dnf install clang curl git openssl-devel```
- macOS: ```brew update && brew install openssl```
- Windows: Refer to this [installation guide](https://docs.substrate.io/v3/getting-started/windows-users/).

### Install Rust and the Rust toolchain

This project uses [`rustup`](https://rustup.rs/) to help manage the Rust toolchain. First install
and configure `rustup`:

```bash
# Install
curl https://sh.rustup.rs -sSf | sh

# Configure
source ~/.cargo/env
```

Finally, Configure the Rust toolchain:

```bash

#### Configure default to the latest stable version:
rustup default stable
rustup update

###F Add the nightly release and the nightly WebAssembly (wasm) targets:
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

#### Verify your installation:
rustc --version
rustup show

```

### Prepare a BHO node

To build the BHO node:

1. Clone the bholdus-chain repository

```
git clone https://github.com/Bholdus/bholdus-chain.git

```
2. Change to the root directory where you compiled the bholdus node

```
cd bholdus-chain
```
3. Start a BHO node

In early stage, we only support Proof of Staked Authority with private authorities. Hence, users can only run RPC node at the moment.

We have two ways to start a BHO node:

1. Building from source

You can clone the repository and try following commands

```
- Run Testnet Node: Phoenix

 cargo build --features with-phoenix-runtime,evm-tracing --release
 ./target/release/bholdus --chain=phoenix --name <INSERT_NAME> --rpc-port 9933 --ws-port 9944 --ws-external --rpc-external --prometheus-external --rpc-cors=all --pruning archive -lruntime=debug

- Run Mainnet Node: Ulas

 cargo build --features with-ulas-runtime,evm-tracing --release
 ./target/release/bholdus --chain=ulas --name <INSERT_NAME> --rpc-port 9933 --ws-port 9944 --ws-external --rpc-external --prometheus-external --rpc-cors=all --pruning archive -lruntime=debug

```
You should always use the --release flag to build optimized artifacts.

Use branch `main` for latest development codebase. However, if you want to build production node, please checkout the tagged release commit. For production build, you can put `SKIP_WASM_BUILD=1` env to disable building wasm version of the runtime.

2. Use pre-built binary

You can use the pre-built binary in the Github Releases

To start a node with a pre-built binary, simply use the commands from `1. Building from source`

#### Multi-Node Local Testnet
If you want to start the multi-node consensus with an authority set of private validators, refer to [our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).


### Benchmarking

To build in benchmarking mode:
```
cargo build --features with-phoenix-runtime --features runtime-benchmarks --release
```

To run benchmarks and output new weight files

```
Example:

./target/release/bholdus benchmark \
    --chain phoenix-dev \               # Configurable Chain Spec
    --execution wasm \          # Always test with Wasm
    --wasm-execution compiled \ # Always used `wasm-time`
    --pallet bholdus-tokens \   # Select the pallet
    --extrinsic '*' \          # Select the benchmark case name, using '*' for all
    --steps 20 \                # Number of steps across component ranges
    --repeat 10 \               # Number of times we repeat a benchmark
    --raw \                     # Optionally output raw benchmark data to stdout
    --output ./pallets/bholdus-tokens/src/weights.rs  # Output results into a Rust file
    --template ./.maintain/frame-weight-template.hbs  # Template file

```


## Connecting the BHO node with a User Interface

Now that we have the bholdus node running, let's connect it with a [Bholdus-JS Apps](https://apps.bholdus.com/) to see it working.

Follow the following steps,
- One the local node is running, open the following in your browser,
```
https://apps.bholdus.com
```
- Select `Local node` and  Click `Switch`. The Bholdus-JS app would be connected to your local node.








