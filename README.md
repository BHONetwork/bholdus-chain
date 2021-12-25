# Bholdus Node

[![Bholdus](https://img.shields.io/badge/Bholdus-brightgreen?logo=Parity%20Substrate)](https://apps.bholdus.com/)

A fresh FRAME-based [Substrate](https://www.substrate.io/) node

## Getting Started

Bholdus Blockchain Network is interoperable, cross-chain with various digital asset economies and Defi Networks such as Binance Smart Chain, Ethereum, and Polkadot.

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

### Prepare a Bholdus node

To build the Bholdus node:

1. Clone the bholdus-chain repository

```
git clone https://github.com/Bholdus/bholdus-chain.git

```
2. Change to the root directory where you compiled the bholdus node

```
cd bholdus-chain
```
3. Start a bholdus node

```
- Run Devnet Node: Phoenix

 cargo build --features with-phoenix-runtime --release
 ./target/release/bholdus --chain=phoenix-dev --name <INSERT_NAME>  --tmp -lruntime=debug

 Or you can use `cargo run`
  cargo run --features with-phoenix-runtime --features runtime-benchmarks --release --name <INSERT_NAME> --chain=phoenix-dev --tmp -lruntime=debug

- Run Testnet Node: Cygnus

 cargo build --features with-cygnus-runtime --release
 ./target/release/bholdus --chain=cygnus-dev --name <INSERT_NAME>  --tmp -lruntime=debug

- Run Mainnet Node: Ulas

 cargo build --features with-ulas-runtime --release
 ./target/release/bholdus --chain=ulas-dev --name <INSERT_NAME>  --tmp -lruntime=debug

```
You should always use the --release flag to build optimized artifacts.

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
    --output ./                 # Output results into a Rust file

```


## Connecting the Bholdus node with a User Interface

Now that we have the bholdus node running, let's connect it with a [Bholdus-JS Apps](https://apps.bholdus.com/) to see it working.

Follow the following steps,
- One the local node is running, open the following in your browser,
```
https://apps.bholdus.com
```
- Select `Local node` and  Click `Switch`. The Bholdus-JS app would be connected to your local node.








