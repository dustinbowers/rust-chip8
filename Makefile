.PHONY: default build release wasm wasm-release

TARGET_DIR := ./target
WASM_TYPE := wasm32-unknown-unknown
HTML_DIR := ./html

default: build

build:
	cargo build

release:
	cargo build --release

wasm:
	cargo build --target $(WASM_TYPE)
	\cp $(TARGET_DIR)/$(WASM_TYPE)/debug/chip8.wasm $(HTML_DIR)

wasm-release:
	cargo build --target $(WASM_TYPE) --release
	\cp $(TARGET_DIR)/$(WASM_TYPE)/release/chip8.wasm $(HTML_DIR)
