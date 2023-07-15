cargo fmt --all
cargo clippy --fix --features rt_tokio --allow-staged
cargo clippy --fix --features event --allow-staged
cargo fmt --all