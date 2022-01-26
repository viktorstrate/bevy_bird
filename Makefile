
.PHONY: web

web:
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --out-dir ./web/target/ --target web ./target/wasm32-unknown-unknown/release/bevy_bird.wasm
	cp -r assets web/
