default: build

all: test build

test:
	cargo test

build:
	cargo build

build-wasm:
	cargo rustc --target wasm32-unknown-unknown --release --crate-type cdylib

optimize: build-wasm
	@echo "Optimization step: install stellar-cli to run stellar contract optimize"

fmt:
	cargo fmt --all

clean:
	cargo clean

