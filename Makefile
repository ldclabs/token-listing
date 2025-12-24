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
	cargo build --release --target wasm32-unknown-unknown -p ic_auction -p token_listing_canister -p tokens_canister -p tokens_canister

# cargo install candid-extractor
build-did:
	candid-extractor target/wasm32-unknown-unknown/release/ic_auction.wasm > src/ic_auction/ic_auction.did
	candid-extractor target/wasm32-unknown-unknown/release/token_listing_canister.wasm > src/token_listing_canister/token_listing_canister.did
	candid-extractor target/wasm32-unknown-unknown/release/tokens_canister.wasm > src/tokens_canister/tokens_canister.did
	dfx generate

gzip-wasm:
	@set -e; \
	for CAN in ic_auction token_listing_canister tokens_canister; do \
    cp "target/wasm32-unknown-unknown/release/$$CAN.wasm" debug/; \
    cp "src/$$CAN/$$CAN.did" "debug/$$CAN.did"; \
    WASM="debug/$$CAN.wasm"; \
    ic-wasm "$$WASM" -o "$$WASM" metadata candid:service -f "debug/$$CAN.did" -v public; \
    ic-wasm "$$WASM" -o "$$WASM" shrink; \
    ic-wasm "$$WASM" -o "$$WASM" optimize O3 --inline-functions-with-loops; \
    gzip "$$WASM"; \
	done
