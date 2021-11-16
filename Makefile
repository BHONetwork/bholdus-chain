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

.PHONY: check-debug-ulas
check-debug-ulas:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-ulas-runtime --release
.PHONY: check-debug-cygnus
check-debug-cygnus:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-cygnus-runtime --release
.PHONY: check-debug-phoenix
check-debug-phoenix:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-phoenix-runtime --release

.PHONY: test-ulas
test-ulas:
	SKIP_WASM_BUILD= cargo test --features with-ulas-runtime --release -- --nocapture
.PHONY: test-cygnus
test-cygnus:
	SKIP_WASM_BUILD= cargo test --features with-cygnus-runtime --release -- --nocapture
.PHONY: test-phoenix
test-phoenix:
	SKIP_WASM_BUILD= cargo test --features with-phoenix-runtime --release -- --nocapture
