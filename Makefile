.PHONY: default build release wasm wasm-release build-test

PACKAGE_NAME = chip8
TARGET_DIR := ./target
WASM_TYPE := wasm32-unknown-unknown
DIST_DIR := ./dist

default: build

build:
	cargo build

release:
	cargo build --release

wasm:
	./wasm-bindgen-macroquad.sh chip8
# 	cargo build --target $(WASM_TYPE)
# 	\cp $(TARGET_DIR)/$(WASM_TYPE)/debug/$(PACKAGE_NAME).wasm $(DIST_DIR)

wasm-release:
	./wasm-bindgen-macroquad.sh chip8 --release
# 	cargo build --target $(WASM_TYPE) --release
# 	\cp $(TARGET_DIR)/$(WASM_TYPE)/release/$(PACKAGE_NAME).wasm $(DIST_DIR)

