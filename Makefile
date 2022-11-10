.PHONY: build

build_binary:
	cargo build --target x86_64-apple-darwin --release
	cargo build --target x86_64-pc-windows-gnu --release
	cargo build --target x86_64-unknown-linux-gnu --release
	cargo build --target x86_64-unknown-linux-musl --release

build: build_binary

permissions:
	chmod 755 target/x86_64-apple-darwin/release/genin
	chmod 755 target/x86_64-unknown-linux-gnu/release/genin
	chmod 755 target/x86_64-unknown-linux-musl/release/genin
