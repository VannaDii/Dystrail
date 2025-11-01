# Standardized CLI workflows for Dystrail

fmt:
    cargo fmt --all

lint:
    cargo fmt -- --check
    cargo check --workspace
    @cargo clippy --workspace --all-targets --all-features -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery
    cargo test --workspace --all --all-features --locked -- --nocapture
    wasm-pack test --headless --chrome dystrail-web
    cargo tarpaulin --packages dystrail-game --fail-under 77

security:
    cargo audit --deny-warnings
    cargo deny check licenses bans advisories sources

build-release:
    cd dystrail-web && trunk build --release
    cargo build --workspace --release

qa:
    cargo run -p dystrail-tester -- --mode both --scenarios all --iterations 100 --report console

validate:
    just lint
    cargo check --workspace --all-targets --all-features
    just tests
    just security
    just build-release
    just qa

check:
    just validate

coverage:
    cargo tarpaulin --packages dystrail-game --fail-under 77
