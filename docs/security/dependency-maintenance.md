# Open dependency maintenance issues

Recorded September 11, 2026. Owner: project maintainers. These are local tracking
issues, not externally posted reports. The repository's `AGENT.md` permits an
audit with documented temporary exceptions and an issue. Both exceptions below
expire **October 11, 2026**. CI checks the date, exact dependency versions and
matching audit/deny configuration before running either advisory gate.

The compatible dependency updates remove the seven previously reported
vulnerabilities and the unsoundness warnings. The unconfigured audit still
reports three maintenance warnings. Those warnings are **not fixed** by these
exceptions. No vulnerability advisory is allowed. New advisories still fail.

## SUPPLY-001

Status: open. Replace the two transitive `bincode` dependencies through an
upstream Yew/gloo release, then remove RUSTSEC-2025-0141 from both configurations
and the exception registry.

- `bincode 1.3.3` arrives through `gloo-worker 0.5.0 → gloo 0.11.0 → Yew`.
- `bincode 2.0.1` arrives through Yew, including the SSR feature used by native
  component tests. Removing SSR would remove useful validation.
- The game does not directly call bincode or gloo worker APIs; saves use JSON.
  This limits current exposure but does not guarantee future maintenance safety.
- [RustSec classifies the advisory as unmaintained, with no patched version](https://rustsec.org/advisories/RUSTSEC-2025-0141.html).
- Inspected Yew master `bfa6c19af971084f9547495388bde9aceeb81aaa` still contains
  bincode. A framework bump alone does not close this issue. No upstream code
  has been vendored or substituted.

## SUPPLY-002

Status: open. Replace `proc-macro-error 1.0.4` through an upstream `yew-macro`
release, then remove RUSTSEC-2024-0370 from both configurations and the registry.

- This is a compiler-side dependency of `yew-macro`; it is not the game's save
  parser or a runtime network handler.
- [RustSec classifies it as unmaintained, with no patched version](https://rustsec.org/advisories/RUSTSEC-2024-0370.html).
- Inspected Yew master still contains this dependency. The previous repository
  exception covered the same advisory, but its root `audit.toml` was not loaded.

The obsolete `paste` exception is removed after updating Thirtyfour. Cargo Audit
now uses `.cargo/audit.toml`; `--file audit.toml` was incorrectly treating the
configuration as a lockfile. `Cargo.lock` is included for reproducible builds.
