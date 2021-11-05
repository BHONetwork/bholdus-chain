.PHONY: run
run:
	cargo run --features with-bholdus-runtime -- --dev --tmp -lruntime=debug

.PHONY: run-benchmark
run-benchmark:
	cargo run --features with-bholdus-runtime --features runtime-benchmarks -- --dev --tmp -lruntime=debug

.PHONY: toolchain
toolchain:
	./scripts/init.sh

.PHONY: build
build:
	SKIP_WASM_BUILD= cargo build --features with-bholdus-runtime

.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-bholdus-runtime


.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --features with-bholdus-runtime -- --nocapture
