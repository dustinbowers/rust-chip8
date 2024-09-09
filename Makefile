.PHONY: default build release wasm wasm-release web-server build-test-web

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

build-test-web-release: wasm-release web-server
