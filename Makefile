.PHONY: run-ulas
run-ulas:
	cargo +nightly run --features with-ulas-runtime --release -- --alice --chain=ulas-dev --tmp -lruntime=debug
.PHONY: run-cygnus
run-cygnus:
	cargo +nightly run --features with-cygnus-runtime --release -- --alice --chain=cygnus-dev --tmp -lruntime=debug
.PHONY: run-phoenix
run-phoenix:
	cargo +nightly run --features with-phoenix-runtime --release -- --alice --chain=phoenix-dev --tmp -lruntime=debug

.PHONY: run-benchmark-ulas
run-benchmark-ulas:
	cargo run --features with-ulas-runtime --features runtime-benchmarks --release -- --alice --chain=ulas-dev --tmp -lruntime=debug
.PHONY: run-benchmark-cygnus
run-benchmark-cygnus:
	cargo run --features with-cygnus-runtime --features runtime-benchmarks --release -- --alice --chain=cygnus-dev --tmp -lruntime=debug
.PHONY: run-benchmark-phoenix
run-benchmark-phoenix:
	cargo run --features with-phoenix-runtime --features runtime-benchmarks --release -- --alice --chain=phoenix-dev --tmp -lruntime=debug

.PHONY: toolchain
toolchain:
	./scripts/init.sh

.PHONY: build-ulas
build-ulas:
	SKIP_WASM_BUILD= cargo build --features with-ulas-runtime --release
.PHONY: build-cygnus
build-cygnus:
	SKIP_WASM_BUILD= cargo build --features with-cygnus-runtime --release
.PHONY: build-phoenix
build-phoenix:
	SKIP_WASM_BUILD= cargo build --features with-phoenix-runtime --release
.PHONY: build-all
build-all:
	cargo build --locked --features with-all-runtime

.PHONY: check-debug-ulas
check-debug-ulas:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-ulas-runtime --release
.PHONY: check-debug-cygnus
check-debug-cygnus:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-cygnus-runtime --release
.PHONY: check-debug-phoenix
check-debug-phoenix:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-phoenix-runtime --release

.PHONY: test-ulas-runtime
test-ulas-runtime:
	SKIP_WASM_BUILD= cargo test --features with-ulas-runtime -- --nocapture
.PHONY: test-cygnus-runtime
test-cygnus-runtime:
	SKIP_WASM_BUILD= cargo test --features with-cygnus-runtime -- --nocapture
.PHONY: test-phoenix-runtime
test-phoenix-runtime:
	SKIP_WASM_BUILD= cargo test --features with-phoenix-runtime -- --nocapture

.PHONY: test-runtimes
test-runtimes:
	SKIP_WASM_BUILD= cargo test --all --features with-all-runtime

.PHONY: test-ulas-benchmarking
test-ulas-benchmarking:
	cargo test --features runtime-benchmarks,with-ulas-runtime --all benchmarking
.PHONY: test-cygnus-benchmarking
test-cygnus-benchmarking:
	cargo test --features runtime-benchmarks,with-cygnus-runtime --all benchmarking
.PHONY: test-phoenix-benchmarking
test-phoenix-benchmarking:
	cargo test --features runtime-benchmarks,with-phoenix-runtime --all benchmarking

.PHONY: test-benchmarking
test-benchmarking:
	cargo test --features runtime-benchmarks --features with-all-runtime --features --all benchmarking

.PHONY: check-ulas-try-runtime
check-ulas-try-runtime:
	SKIP_WASM_BUILD= cargo check --features try-runtime,with-ulas-runtime
.PHONY: check-cygnus-try-runtime
check-cygnus-try-runtime:
	SKIP_WASM_BUILD= cargo check --features try-runtime,with-cygnus-runtime
.PHONY: check-phoenix-try-runtime
check-phoenix-try-runtime:
	SKIP_WASM_BUILD= cargo check --features try-runtime,with-phoenix-runtime

.PHONY: check-try-runtime
check-try-runtime:
	SKIP_WASM_BUILD= cargo check --features try-runtime --features with-all-runtime