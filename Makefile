.PHONY: default build release wasm wasm-release build-test-web

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

wasm-release:
	./wasm-bindgen-macroquad.sh chip8 --release

web-server:
	basic-http-server ./dist

build-test-web: wasm web-server
