.PHONY: build test run clean

build:
	cargo build --release

test:
	cargo test

run:
	cargo run -- $(ARGS)

clean:
	cargo clean

