.PHONY: build default

default: ;

install-cargo:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |\
		sh -s -- -y --profile default --default-toolchain 1.65.0

build_binary:
	cargo build --target x86_64-apple-darwin --release
	cargo build --target x86_64-pc-windows-gnu --release
	cargo build --target x86_64-unknown-linux-gnu --release
	cargo build --target x86_64-unknown-linux-musl --release

build:
	. ~/.cargo/env && \
	cargo build

permissions:
	find target -type f -name genin -exec chmod 755 {} \;

install:
	mkdir -p $(DESTDIR)/usr/bin && \
	install -m 0755 target/*/genin $(DESTDIR)/usr/bin/genin
