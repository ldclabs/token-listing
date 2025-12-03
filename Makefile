BUILD_ENV := rust

.PHONY: build-wasm build-did

lint:
	@cargo fmt
	@cargo clippy --all-targets --all-features

fix:
	@cargo clippy --fix --workspace --tests

test:
	@cargo test --workspace -- --nocapture

# cargo install ic-wasm
build-wasm:
	cargo build --release --target wasm32-unknown-unknown --package icp_ccca

# cargo install candid-extractor
build-did:
	candid-extractor target/wasm32-unknown-unknown/release/icp_ccca.wasm > src/icp_ccca/icp_ccca.did
	dfx generate
