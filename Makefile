.PHONY: build pack

build:
	cargo build --target x86_64-apple-darwin --release
	cargo build --target x86_64-pc-windows-gnu --release
	cargo build --target x86_64-unknown-linux-gnu --release
	cargo build --target x86_64-unknown-linux-musl --release

pack:
	zip -j genin-${CI_COMMIT_TAG}-darwin-amd64.zip target/x86_64-apple-darwin/release/genin
	zip -j genin-${CI_COMMIT_TAG}-windows-amd64.zip target/x86_64-pc-windows-gnu/release/genin.exe
	zip -j genin-${CI_COMMIT_TAG}-linux-gnu-amd64.zip target/x86_64-unknown-linux-gnu/release/genin
	zip -j genin-${CI_COMMIT_TAG}-linux-musl-amd64.zip target/x86_64-unknown-linux-musl/release/genin
	mkdir -p target/package
	mv *.zip target/package/
