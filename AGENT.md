# AGENT.md — Codex Operating Instructions (Rust + Yew, A11y + i18n Required)

IMPORTANT: NEVER TAKE SHORT CUTS! ONLY USE TERMINAL COMMANDS AS A LAST RESORT.

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
- **serde** + **serde_wasm_bindgen** (data).
- **thiserror** / **anyhow** (errors).
- **yew-style-inliner** or CSS pipeline via Trunk (for SSR-ish critical path styles if needed).

> For date/number formatting, prefer the browser's **Intl** API through `web-sys` bindings to keep WASM small; use the custom JSON-based i18n system for message translations and variable interpolation. (Rust + Yew, A11y + i18n Required)

You are an implementation agent working in a Rust + Yew web app repository. Your output MUST be production-ready, accessible (WCAG 2.2 AA), localized (multiple languages incl. RTL), and conform to Rust best practices.

If any requirement conflicts, **fail the pipeline** and open a TODO with remediation steps.

---

## 0 - Non-Negotiables (Gates)

- ✅ **Accessibility:** WCAG 2.2 AA across pages, keyboard-only usable, visible focus, trap-free modals, correct ARIA, color-contrast ≥ 4.5:1.
- 🌍 **Internationalization:** At least `en`, `it`, `es`, and one RTL (e.g., `ar`). Runtime language switcher; persist locale; pluralization & interpolation; RTL flipping; number/date formatting.
- 🦀 **Rust Quality:** `cargo clippy -- -D warnings -W clippy::pedantic`, `rustfmt`, `cargo test`, `cargo audit`, `cargo deny check` all pass. No unwraps in UI code-paths (use `?` or safe error UX).
- 🔒 **Security/Supply:** No yanked crates; `Cargo.lock` checked in; audit clean or documented (with temporary allow + issue).
- 🧪 **Tests:** Unit (wasm-bindgen-test), i18n snapshot checks for each locale.
- 🛠️ **Reproducible Build:** `rust-toolchain.toml` pins versions; Trunk build; wasm-opt `-Oz`. CI must build release artifacts.
- 🏃🏻‍♀️ **Running Commands:** Never used the terminal, unless it's for **Rust Quality** commands (`cargo`, `rustfmt`, etc.), always run commands using your MCP accessible tools.
- 💇🏽‍♀️ **Styling Rules:** Never use `!important` or other workarounds.

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
- **serde** + **serde_wasm_bindgen** (data).
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
├─ Trunk.toml
├─ index.html
├─ static/                 # favicons, manifest, images
├─ i18n/
│  ├─ en.json
│  ├─ it.json
│  ├─ es.json
│  └─ ar.json               # RTL example
├─ src/
│  ├─ main.rs
│  ├─ app.rs
│  ├─ router.rs
│  ├─ i18n.rs
│  ├─ a11y.rs             # focus mgmt, aria helpers, traps
│  ├─ components/
│  │  ├─ header.rs        # language switcher, skip link
│  │  ├─ footer.rs
│  │  ├─ button.rs        # keyboard & aria complete
│  │  ├─ modal.rs         # focus trap, esc close, aria-modal
│  │  └─ form/
│  │     ├─ field.rs      # label/aria-describedby, errors
│  │     └─ text_input.rs
│  └─ pages/
│     ├─ home.rs
│     └─ settings.rs
└─ tests/
├─ wasm/
   └─ app_tests.rs        # wasm-bindgen-test
```

---

## 3 - Commands (scripts)

| Task | Command |
| --- | --- |
| Dev server | `trunk serve --open` |
| Release build | `trunk build --release` |
| Format | `cargo fmt --all` |
| Lint | `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` |
| Unit tests (wasm) | `wasm-pack test --headless --chrome` **or** `cargo test -p <crate>` if using wasm-bindgen-test harness |
| Security audit | `cargo audit` |
| License/dep policy | `cargo deny check` |

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

- Store messages in `i18n/<locale>.json`. Keys are **namespaced** (`app.nav.home`, `form.error.required`).
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
- No `.unwrap()` or `.expect()` in UI path; use `ok_or_else` + user-safe fallback.

---

## 5 - Key Files (Sketches)

### 5.1 - `src/main.rs`

```rust
#![forbid(unsafe_code)]
use yew::prelude::*;
mod app;
fn main() {
    console_error_panic_hook::set_once(); // dev only (cfg)
    yew::Renderer::<app::App>::new().render();
}
```

### 5.2 - src/i18n.rs (loader + context)

```rust
use serde_json::Value;
use std::collections::HashMap;

pub struct I18nBundle {
    pub lang: String,
    pub rtl: bool,
    translations: Value,
    fallback: Value,
}

impl I18nBundle {
    pub fn t(&self, key: &str) -> String { /* resolve string or fallback */ }
    pub fn tr(&self, key: &str, args: Option<&HashMap<&str, &str>>) -> String { /* resolve with variable interpolation */ }
    pub fn set_lang(&mut self, lang: &str) { /* rebuild bundle, set rtl, update DOM lang/dir */ }
    pub fn is_rtl(&self) -> bool { self.rtl }
}
```

### 5.3 - src/a11y.rs (focus utils)

```rust
pub fn restore_focus(prev_id: &str) { /* querySelector + focus via web-sys */ }
pub fn trap_focus_in(container_id: &str) { /* on keydown Tab, cycle focusables */ }
```

### 5.4 - src/components/modal.rs (behavioral checklist)

- Add aria roles/labels.
- Focus trap activate on open; restore focus on close.
- Close on Esc, overlay click optional.

⸻

## 6 - i18n Message Example (i18n/en.json)

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
