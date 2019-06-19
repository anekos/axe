
release:
	cargo build --release --features notification

# NO notifications
release-silent:
	cargo build --release
