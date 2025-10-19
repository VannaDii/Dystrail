# Standard checks for both frontend and backend
check:
    cargo fmt -- --check
    cargo check --workspace
    @cargo clippy --workspace --all-targets --all-features -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple_crate_versions
    cargo test --workspace --all --all-features --locked -- --nocapture
    cargo tarpaulin --packages=dystrail-game --packages=dystrail-web

coverage:
    cargo tarpaulin --packages=dystrail-game --packages=dystrail-web
