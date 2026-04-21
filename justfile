default: check

check:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features

fmt:
    cargo fmt --all

test:
    cargo test --all-features

build:
    cargo build --release

install:
    cargo install --path . --locked

bacon:
    bacon

review:
    cargo insta review
