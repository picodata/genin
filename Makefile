.PHONY: build default

default: ;

install-cargo:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |\
		sh -s -- -y --profile default --default-toolchain 1.85.0

build_binary: build_apple build_windows build_gnu build_musl

build_apple:
	cargo build --target x86_64-apple-darwin --release

build_windows:
	cargo build --target x86_64-pc-windows-gnu --release

build_gnu:
	cargo build --target x86_64-unknown-linux-gnu --release

build_musl:
	cargo build --target x86_64-unknown-linux-musl --release

build:
	. ~/.cargo/env && \
	cargo build

permissions:
	find target -type f -name genin -exec chmod 755 {} \;
	chown -R $(id -u):$(id -g) .

install:
	mkdir -p $(DESTDIR)/usr/bin && \
	install -m 0755 target/*/genin $(DESTDIR)/usr/bin/genin
