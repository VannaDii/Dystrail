# Standardized CLI workflows for Dystrail

fmt:
    cargo fmt --all

lint:
    cargo fmt -- --check
    cargo check --workspace
    cargo clippy --workspace --all-targets --all-features -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
    cargo test --workspace --all --all-features --locked -- --nocapture
    wasm-pack test --headless --chrome dystrail-web
    cargo tarpaulin --packages dystrail-game --fail-under 77

security:
    cargo audit --file audit.toml --deny warnings
    cargo deny check licenses bans advisories sources

build-release:
    cd dystrail-web && trunk build --release
    cargo build --workspace --release

serve-web:
    # Run the web UI with live reload/watch
    cd dystrail-web && trunk serve --open --watch

qa:
    cargo run -p dystrail-tester -- --mode logic --scenarios real-game --iterations 1000 --report console

tests:
    cargo test --workspace --all --all-features --locked -- --nocapture

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
