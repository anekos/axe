
release:
	cargo build --release --features notification

release-musl:
	cross build --release --target x86_64-unknown-linux-musl

# NO notifications
release-silent:
	cargo build --release
