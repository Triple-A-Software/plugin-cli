build-rust:
    mkdir -p bin
    cargo build --release --target x86_64-unknown-linux-gnu
    cp target/x86_64-unknown-linux-gnu/release/plugin-cli bin/plugin-cli-linux-x86_64
    # cargo build --release --target x86_64-apple-darwin
    # cp target/x86_64-apple-darwin/release/plugin-cli bin/plugin-cli-darwin-x86_64
    # cargo build --release --target x86_64-pc-windows-msvc
    # cp target/x86_64-pc-windows-msvc/release/plugin-cli.exe bin/plugin-cli-windows-x86_64
    # cargo build --release --target aarch64-unknown-linux-gnu
    # cp target/aarch64-unknown-linux-gnu/release/plugin-cli bin/plugin-cli-linux-aarch64
