cargo tarpaulin -o Html --ignore-tests --skip-clean --exclude-files example/ target/ &&
cargo doc &&
cargo clippy --fix -Z unstable-options --allow-dirty &&
cargo fmt
