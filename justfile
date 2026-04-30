default:
	just debug

build:
	toolbox run -c dflt cargo build --release

build-win:
	toolbox run -c dflt cargo build --release --target x86_64-pc-windows-gnu

test:
	cd turbo-csv && RUST_LOG=flugs,app_core=debug RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo test turbo-csv

release:
	just build
	cp ./target/release/flugs ~/.local/bin

release-win:
	just build-win
	cp ./target/x86_64-pc-windows-gnu/release/flugs.exe ~/ownCloud/Exchange/DS_Exchange/Software/

debug:
	RUST_LOG=flugs,app_core=debug RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo run --release
