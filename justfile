default:
	just debug

build:
	cargo build --release

test:
	cd turbo-csv && RUST_LOG=flugs,app_core=debug RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo test turbo-csv


debug:
	RUST_LOG=flugs,app_core=debug RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo run --release
