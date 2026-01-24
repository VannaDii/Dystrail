# Standardized CLI workflows for Dystrail

fmt:
    cargo fmt --all

lint:
    cargo fmt -- --check
    cargo check --workspace
    cargo clippy --workspace --all-targets --all-features -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
    cargo test --workspace --all --all-features --locked -- --nocapture
    wasm-pack test --headless --chrome dystrail-web
    just coverage

security:
    cargo audit --file audit.toml --deny warnings
    cargo deny check licenses bans advisories sources

build-release:
    cd dystrail-web && NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/
    cargo build --workspace --release

serve-web:
    # Run the web UI with live reload/watch
    cd dystrail-web && PUBLIC_URL=/play trunk serve --open --watch . --public-url /play/ --port 8081

qa:
    cargo run -p dystrail-tester -- --mode logic --scenarios real-game --iterations 1000 --report console

tests:
    cargo test --workspace --all --all-features -- --nocapture

validate:
    just lint
    cargo check --workspace --all-targets --all-features
    just tests
    just security
    just build-release

check:
    just validate

coverage:
    cargo tarpaulin --workspace --follow-exec --fail-under 100

# Docs
docs-build:
    cd docs && mdbook build

docs-serve:
    cd docs && mdbook serve --open
