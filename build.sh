cargo tarpaulin -o Html --ignore-tests --exclude-files example/ && 
cargo doc &&
cargo +nightly clippy --fix -Z unstable-options --allow-dirty &&
rustfmt src/*.rs
