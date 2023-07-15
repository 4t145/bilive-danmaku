git add .
cargo fmt --all
git add .
cargo clippy --fix --features rt_tokio --allow-staged
git add .
cargo clippy --fix --features event --allow-staged
git add .
cargo fmt --all
git add .