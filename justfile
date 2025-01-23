debug:
	RUST_LOG=csv_plotter,app_core=debug RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo run --release
