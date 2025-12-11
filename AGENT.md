# AGENT.md — Codex Operating Instructions (Rust + Yew, A11y + i18n Required)

IMPORTANT: NEVER TAKE SHORT CUTS! Document and run the required CLI workflows for every change.

- **Stable Rust** (latest stable channel pinned).
- **Yew** (component framework).
- **yew_router** (routing).
- **serde** + **serde_json** (JSON-based i18n messages with variable interpolation).
- **gloo** / **web-sys** (browser interop).
- **wasm-bindgen-test** (wasm tests).
- **console_error_panic_hook** (dev diagnostics only).
- **trunk** (build/serve).
- **binaryen/wasm-opt** (size/perf).
- **cargo-audit**, **cargo-deny** (security/licensing).
- **serde** + **serde-wasm-bindgen** (data).
- **thiserror** / **anyhow** (errors).
- **yew-style-inliner** or CSS pipeline via Trunk (for SSR-ish critical path styles if needed).

> For date/number formatting, prefer the browser's **Intl** API through `web-sys` bindings to keep WASM small; use the custom JSON-based i18n system for message translations and variable interpolation. (Rust + Yew, A11y + i18n Required)

You are an implementation agent working in a Rust + Yew web app repository. Your output MUST be production-ready, accessible (WCAG 2.2 AA), localized (multiple languages incl. RTL), and conform to Rust best practices.

If any requirement conflicts, **fail the pipeline**, pause implementation, and document the blocking issue for the user—do not add TODO/FIXME comments.

---

## 0 - Non-Negotiables (Gates)

- ✅ **Accessibility:** WCAG 2.2 AA across pages, keyboard-only usable, visible focus, trap-free modals, correct ARIA, color-contrast ≥ 4.5:1.
- 🌍 **Internationalization:** At least `en`, `it`, `es`, and one RTL (e.g., `ar`). Runtime language switcher; persist locale; pluralization & interpolation; RTL flipping; number/date formatting.
- 🦀 **Rust Quality:** `cargo clippy -- -D warnings -W clippy::pedantic`, `rustfmt`, `cargo test`, `cargo audit`, `cargo deny check` all pass. No unwraps in UI code-paths (use `?` or safe error UX). Lint suppressions are not permitted except `clippy::multiple-crate-versions`, which is allowed until upstream dependencies converge.
- 🚫 **No Suppression Flags:** Do not use any CLI or tooling flags that suppress diagnostics (e.g., `--ignore-filename-regex '(dystrail-(game|web|tester))'`, `--allow-dirty`, `--allow-staged`). Tool outputs must remain fully visible. For dependency ecosystems with unavoidable duplicates (e.g., multi-version crates from upstream), multiple crate versions are allowed via `-Aclippy::multiple-crate-versions`.
- 🔒 **Security/Supply:** No yanked crates; `Cargo.lock` checked in; audit clean or documented (with temporary allow + issue).
- 🧪 **Tests:** Unit (wasm-bindgen-test), i18n snapshot checks for each locale.
- 🛠️ **Reproducible Build:** `rust-toolchain.toml` pins versions; Trunk build; wasm-opt `-Oz`. CI must build release artifacts.
- 🏃🏻‍♀️ **Running Commands:** Use the CLI workflows defined in this AGENT, the `Justfile`, and the README; record every validation command you execute.
- 💇🏽‍♀️ **Styling Rules:** Never use `!important` or other workarounds.
- 🧱 **Read-Only Handling:** Detect sandbox/write permissions before editing (e.g., `test -w .` via trusted shell). If you are in read-only mode, immediately inform the user, request write access, and restrict yourself to inspection commands until access is granted.
- 📋 **TODO Policy:** Never introduce TODO/FIXME notes. When you encounter existing TODOs, remove them by implementing the fix or escalating the conflict—code must not contain unresolved TODO markers.
- 🚫 **No Vendoring:** Do not vendor upstream crates or copy external sources into this repository. Dependency tweaks must be handled through proper version bumps or patches approved by the user.

---

## 1 - Toolchain & Core Libraries

Pin versions in `rust-toolchain.toml` and `Cargo.toml`:

- **Stable Rust** (latest stable channel pinned).
- **Yew** (component framework).
- **yew_router** (routing).
- **gloo** / **web-sys** (browser interop).
- **wasm-bindgen-test** (wasm tests).
- **console_error_panic_hook** (dev diagnostics only).
- **trunk** (build/serve).
- **binaryen/wasm-opt** (size/perf).
- **cargo-audit**, **cargo-deny** (security/licensing).
- **serde** + **serde-wasm-bindgen** (data).
- **thiserror** / **anyhow** (errors).
- **yew-style-inliner** or CSS pipeline via Trunk (for SSR-ish critical path styles if needed).

> For date/number formatting, prefer the browser’s **Intl** API through `web-sys` bindings to keep WASM small.

---

## 2 - Repository Layout

```text
.
├─ .github/workflows/ci.yml
├─ AGENT.md                # (this file)
├─ Cargo.toml
├─ Cargo.lock
├─ rust-toolchain.toml
├─ Justfile
├─ dystrail-game/
├─ dystrail-web/
│  ├─ Cargo.toml
│  ├─ Trunk.toml
│  ├─ index.html
│  ├─ static/                 # favicons, manifest, images, JSON data
│  ├─ i18n/                   # translations (20 locales)
│  └─ src/
│     ├─ lib.rs               # WASM entrypoint
│     ├─ app/                 # bootstrap, routing glue, app state, views
│     ├─ router.rs
│     ├─ pages/               # boot, persona, outfitting, menu, travel, camp, encounter, boss, result, 404
│     ├─ components/          # header/footer/button/modal + ui/*
│     │  └─ ui/               # travel panel, pace/diet panel, camp panel, vehicle status, result screen, settings dialog, etc.
│     ├─ i18n/                # bundle/render/format/locales
│     ├─ a11y.rs              # focus mgmt, aria helpers, traps, contrast toggle
│     ├─ dom.rs
│     ├─ game.rs
│     ├─ input.rs
│     └─ paths.rs
├─ dystrail-tester/
└─ docs/
```

---

## 3 - Commands (scripts)

| Task            | Command              |
| --------------- | -------------------- |
| Format          | `just fmt`           |
| Lint            | `just lint`          |
| Workspace tests | `just tests`         |
| Security audit  | `just security`      |
| Release build   | `just build-release` |
| QA sweeps       | `just qa`            |
| Full validation | `just validate`      |
| Dev server      | `just serve-web`     |

> CI must run all above; any failure blocks merge.

---

## 4 - Implementation Rules

### 4.1 - Accessibility (enforceable)

- Provide a **Skip to content** link as the first focusable element.
- Maintain a visible custom **focus ring** on all interactive controls.
- All interactive components must be **keyboard-operable** (Tab/Shift+Tab/Enter/Space/Escape/Arrows as appropriate).
- **Modal**: trap focus inside, restore focus on close, close on `Esc`, `aria-modal="true"`, `role="dialog"`, label via `aria-labelledby`.
- **Forms**: `<label for>` associations; error text bound via `aria-describedby`; state via `aria-invalid`.
- **Images**: meaningful `alt`; decorative images `role="presentation"` or empty alt.
- **Links vs Buttons**: navigation uses links; actions use buttons.
- **Color contrast** ≥ 4.5:1 (normal text), ≥ 3:1 (UI icons/graphics). Provide a high-contrast theme toggle.
- No content on hover/focus that traps or times out; meet **2.2.6** (dragging movements not required).
- Use **semantic landmarks**: `<header>`, `<nav>`, `<main>`, `<footer>`, `<section>`.

### 4.2 - Internationalization & Localization

- Store messages in `dystrail-web/i18n/<locale>.json`. Keys are **namespaced** (`app.nav.home`, `form.error.required`).
- Implement a **language switcher** in `Header`; saves preference (localStorage) and updates `<html lang>` and `dir="rtl"` for RTL locales.
- Support **variable interpolation** via JSON templates with `{{var}}` and `{var}` syntax.
- Use browser **Intl** for number/date formatting; never hardcode formats.
- **Do not** concatenate translated strings; always pass variables to the translation functions.
- Provide `ar` (RTL) and verify layout flips (flex direction, icons, carets). Wrap where needed with logical CSS (e.g., `margin-inline-start`).

### 4.3 - Code Quality & Architecture

- Root `#![forbid(unsafe_code)]` for the app crate; justify any required `unsafe` in isolated modules with doc comments and tests.
- Prefer **function components** with typed props; avoid global mutable state.
- State: Yew hooks or a small centralized store (e.g., Yew context). Keep it minimal.
- Routing via `yew_router` with typed routes; 404 page present.
- Errors: typed errors (`thiserror`), surfaced to the UI with non-blocking toasts/alerts; never panic in user flows.
- Side effects isolated; add unit tests for pure functions.
- Inject dependencies (configs, data loaders, RNG, storage/IO) through constructors/parameters; modules must not instantiate their own dependencies so they remain fully testable.
- No `.unwrap()` or `.expect()` in UI path; use `ok_or_else` + user-safe fallback.

### 4.4 - Lint Enforcement

- Remove every lint suppression attribute (`#[allow(...)]`, `#![allow(...)]`, `#[expect(...)]`, etc.) encountered in the codebase; treat their presence as a blocker.
- Do not introduce new suppressions. If clippy fails, refactor the code or split functions/components until it passes without ignores.
- Record any legacy suppression you eliminate in the changelog/PR notes for traceability.

### 4.5 - TODO & Debt Policy

- Eliminate existing `TODO`, `FIXME`, and similar debt markers when you touch a file; convert them into completed code or linked issue references provided by the user.
- Reject changes that would add new debt markers; instead, communicate blockers directly to the user and halt work until resolved.

---

## 5 - Key Files (Sketches)

### 5.1 - `dystrail-web/src/lib.rs`

```rust
#![forbid(unsafe_code)]
use wasm_bindgen::prelude::*;

pub mod app;
pub mod a11y;
pub mod i18n;
// ...

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    // seed DOM lang/dir + high-contrast from saved prefs
    crate::i18n::set_lang(&crate::i18n::current_lang());
    if crate::a11y::high_contrast_enabled() {
        crate::a11y::set_high_contrast(true);
    }
    yew::Renderer::<app::App>::new().render();
}
```

### 5.2 - `dystrail-web/src/i18n/` (loader + context)

```rust
// mod.rs
mod bundle;
mod format;
mod locales;
mod render;

pub use bundle::{I18nBundle, current_lang, is_rtl, set_lang};
pub use format::{fmt_currency, fmt_date_iso, fmt_number, fmt_pct};
pub use locales::{LocaleMeta, locales};
pub use render::{t, tr};

// bundle.rs
use serde_json::Value;
use std::cell::RefCell;

pub struct I18nBundle {
    pub lang: String,
    pub rtl: bool,
    pub translations: Value,
    pub fallback: Value,
}
```

### 5.3 - `dystrail-web/src/a11y.rs` (focus utils)

```rust
pub fn restore_focus(prev_id: &str) { /* querySelector + focus via web-sys */ }
pub fn trap_focus_in(container_id: &str) { /* on keydown Tab, cycle focusables */ }
```

### 5.4 - src/components/modal.rs (behavioral checklist)

- Add aria roles/labels.
- Focus trap activate on open; restore focus on close.
- Close on Esc, overlay click optional.

⸻

## 6 - i18n Message Example (`dystrail-web/i18n/en.json`)

```json
{
  "app": {
    "title": "Rust + Yew Demo"
  },
  "nav": {
    "home": "Home",
    "settings": "Settings"
  },
  "greeting": "Hello, {{name}}!",
  "items": {
    "count": {
      "zero": "No items",
      "one": "1 item",
      "other": "{{count}} items"
    }
  }
}
```

Example RTL (i18n/ar.json) mirrors keys and uses culturally appropriate phrasing. Ensure caret icons and layout flip using logical properties (padding-inline-start).

⸻

## 7 - Testing Strategy

### 7.1 - Unit (WASM)

- Use wasm-bindgen-test for component logic where feasible.
- Pure functions: standard cargo test.

### 7.2 E2E + Accessibility

- Verify keyboard tab order reaches all interactive elements; confirm visible focus.
- Screenshot diffs to ensure RTL flip isn’t regressing layout.

### 7.3 - i18n Coverage

- Parse all .json keys and fail if any key is missing in any shipped locale.
- Render smoke test per locale to catch runtime resolution issues.

⸻

## 8 - CI (GitHub Actions) Required Jobs

1. build: trunk build --release (installs wasm-bindgen-cli, wasm-opt).
2. lint: cargo fmt -- --check, cargo clippy -- -D warnings -W clippy::pedantic.
3. security: cargo audit, cargo deny check.
4. tests-unit: cargo test + wasm-bindgen-test.
5. tests-e2e-a11y: npm ci && npx playwright install --with-deps && npx playwright test.
6. i18n-coverage: custom Rust/Node script to assert all .json keys exist per locale.

Any failure blocks merge. CI must run on PRs and default branch.

⸻

## 9 - Performance Budget

- Enforce wasm-opt -Oz.
- Target initial load < 250KB gzipped WASM for the demo shell (excluding images).
- Lazy-load heavy pages (route-based code splitting where practical).
- Inline critical CSS via Trunk; defer non-critical assets.

⸻

## 10 - Definition of Done (checklist)

- All CI jobs pass.
- Keyboard-only run-through recorded in CI (video) with visible focus.
- Locale switcher persists choice; lang and dir attributes correct.
- ar (RTL) layout visually verified; icons/chevrons flip logically.
- clippy (pedantic), rustfmt, cargo audit, cargo deny clean.
- No unwrap/expect on UI codepaths.
- Error states are localized and announced to AT (e.g., role="alert").
- Release artifact published from CI.

⸻

## 11 - Coding Conventions

- Modules: snake_case; Components: PascalCase with Props structs.
- Public APIs fully documented with rustdoc; examples where sensible.
- Avoid lifetimes in props unless necessary; prefer AttrValue for strings.
- Keep components small (<150–200 lines); extract behaviors into helpers.

⸻

## 12 - PR Hygiene

- Each PR includes: what changed, screenshots/Loom for a11y flows, locales touched.
- Add FIXME: only with linked issue; otherwise, open an issue and reference it.

⸻

## 13 - Stretch (optional, nice-to-have)

- PWA manifest + service worker (offline shell).
- Color scheme toggle (system / light / dark / high-contrast).
- Telemetry (privacy-respecting, opt-in) for a11y errors in production.

⸻

## 14) File & Module Organization (small, cohesive units)

The goal is that **no single file becomes a grab-bag**. Files must stay **small, cohesive, and named for what they own**. If you’re scrolling forever or adding unrelated types “because they’re nearby,” you’re breaking this rule.

### 14.1 General rules

- **Single responsibility per file**

  - Each `*.rs` file must have **one primary responsibility** along a clear axis:
    - by layer (`domain`, `app`, `http`, `infra`, `tasks`, `telemetry`, `config`, etc.), or
    - by feature/vertical (`torrents`, `setup`, `dashboard`, `indexers`), or
    - by type kind (`requests`, `responses`, `errors`, `extractors`, `rate_limit`, etc.).
  - If you can’t summarise the file in a single sentence without “and also,” it’s probably doing too much.

- **File size guidance**

  - Target **≤ ~300–400 non-test LOC per file** for production code.
  - Hitting `clippy::too_many_lines` is treated as a **design smell**, not a lint to be silenced. Fix it by:
    - extracting helpers into private functions,
    - moving cohesive logic into a dedicated module, or
    - splitting the file along a clear responsibility boundary.
  - Test modules may be larger, but if a single `tests` module starts to sprawl, split into `mod something_tests;` files under `tests/`.

- **Naming must reflect contents**
  - A file name must clearly describe what it owns. Some canonical patterns:
    - `api.rs` – API trait / router wiring for a feature or service.
    - `requests.rs` / `responses.rs` – transport DTOs for HTTP.
    - `errors.rs` – error enums/types for that module.
    - `state.rs` – module-local state structs, not global grab-bags.
    - `auth.rs`, `rate_limit.rs`, `sse.rs`, `health.rs` – behaviorally scoped modules.
  - If someone can’t guess what’s inside from the filename, rename or split it.

### 14.2 Types-per-file rules (“like-kind” only)

- **Allowed:** multiple types of the same “kind” in a single file when the name reflects that:

  - `responses.rs` may contain all HTTP response shapes for a given area:
    - `DashboardResponse`, `HealthResponse`, `FullHealthResponse`, etc.
  - `errors.rs` may hold `ApiError`, `DomainError`, helper structs like `ErrorRateLimitContext`, as long as they are **error-centric and local** to that module.
  - `rate_limit.rs` may hold `RateLimiter`, `RateLimitStatus`, `RateLimitSnapshot`, `RateLimitError`, plus helpers.

- **Not allowed:** mixing unrelated kinds in one file:

  - Do **not** define API traits, HTTP handlers, router construction, DTOs, auth extractors, rate limiting, OpenAPI persistence, and test harnesses all in a single `lib.rs` or `api.rs`.
  - Do **not** put domain types, HTTP DTOs, and infra adapters in one file “for convenience.”
  - If two types **would normally live in different folders** (`domain/`, `http/`, `infra/`, `telemetry/`), they must **not** share a file.

- **Like-kind rule of thumb**
  - If all types would be described with the same suffix in docs (“…response types”, “…request types”, “…rate limiting primitives”, “…auth extractors”), they can share a file.
  - If you’d naturally split the sentence (“this file has API traits, response types, and the whole router”), it must be split.

### 14.3 `lib.rs` and `main.rs` constraints

- **`lib.rs`**

  - `lib.rs` is for crate docs (`//!`), `pub mod` declarations, **light** re-exports, and minimal bootstrap glue (e.g., `#[wasm_bindgen(start)]` that sets lang/dir/high-contrast before rendering).
  - Do not park feature logic, UI components, or large state machines in `lib.rs`; move them into named modules (`app/`, `components/`, `pages/`, etc.) and re-export as needed.

- **`main.rs`** (only if a crate has one)
  - Stays a **thin bootstrap**: parse config/CLI, initialize telemetry, wire concrete implementations, then call a `run()`/`bootstrap()` in a dedicated module.
  - No business logic, HTTP handlers, or domain types in `main.rs`.

### 14.4 Module hierarchy & layering

- **Respect crate archetypes (Section 18)** at the directory level and mirror that at the file level:

  - `http/`:
    - `router.rs` – route wiring.
    - `handlers/` – one file per feature/vertical (e.g. `torrents.rs`, `setup.rs`, `health.rs`, `dashboard.rs`).
    - `dto/` – `requests.rs`, `responses.rs`, `errors.rs`.
    - `auth.rs`, `rate_limit.rs`, `sse.rs`, `middleware.rs` – cross-cutting concerns.
  - `app/`:
    - `services/` – orchestration per use-case (`torrents_service.rs`, `setup_service.rs`).
    - No HTTP or transport types here; operate on domain types.
  - `domain/`:
    - `model/` – core types per concept (`torrent.rs`, `config.rs`).
    - `policy/` – rules/decisions in dedicated files.
    - `service/` – pure services per domain concern.

- **Tests and support code**
  - Unit tests local to a module live in the same file in `#[cfg(test)] mod tests { … }`, or in a dedicated `modname_tests.rs` if they get large.
  - Shared test helpers belong in a test support crate (`dystrail-test-support`) or clearly named `tests/fixtures.rs`, **never** in production modules.

### 14.5 Refactoring triggers (when you MUST split a file)

You **must** split or reorganize a file when any of the following are true:

1. Clippy complains about `too_many_lines` and you’re tempted to silence it.
2. The file defines:
   - a long-lived state struct (`ApiState`) **and**
   - a server wrapper (`ApiServer`) **and**
   - HTTP handlers **and**
   - middleware **and/or**
   - DTOs and helper types.
3. A reviewer or your future self struggles to find where a given behaviour lives (“where is rate limiting implemented?”, “where is SSE filtered?”).
4. You find yourself using comments like `// region: X` to mentally group sections — each “region” probably deserves a module.

At each trigger, **split by responsibility** and ensure file names and paths reflect the new structure. Update `lib.rs`/`mod.rs` docs to describe the layout after the change.

_This section is normative. If a file organization choice conflicts with 19.x, reorganize the code to comply rather than weakening lints or adding grab-bag files._
