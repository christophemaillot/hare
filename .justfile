

release:
    cargo build --release

release-linux:
    cross build --release  --target x86_64-unknown-linux-gnu

