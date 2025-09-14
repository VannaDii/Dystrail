# Dystrail — AI Coding Agent Instructions

## Agent Rules
- You are an AI coding assistant for the Dystrail project, a Rust+Yew WASM web game.
- Your detailed instructions are available at `../AGENT.md`.

## Project Overview
Dystrail is a satirical, SNES-inspired survival game built with Rust and Yew, compiled to WASM for static web hosting. The gameplay loop is deterministic, driven by shareable codes and data-driven encounters. No backend or network services are present.

## Architecture & Key Files
- **src/app.rs**: Main app logic, boot overlays, and routing to UI panels.
- **src/components/ui.rs**: UI components (Stats, Share Code Bar, Mode Select, Result Screen).
- **src/game/state.rs**: Game state model (stats, mode, seed, progress). Implements deterministic runs via share codes.
- **src/game/data.rs**: Loads and parses encounter data from `assets/data/game.json`.
- **assets/data/game.json**: Defines encounters and choices; extendable for new content.
- **Trunk.toml**: Configures Trunk for WASM builds and serving.
- **Cargo.toml**: Rust dependencies; Yew, WASM, gloo, serde, regex, etc.

## Developer Workflow
- **Build & Serve:**
  ```bash
  rustup target add wasm32-unknown-unknown
  cargo install trunk
  trunk serve --open
  ```
- **No backend:** All game logic and assets are client-side. No server integration.
- **Extend encounters:** Add new events to `assets/data/game.json` and update models in `src/game/data.rs`.
- **UI/State updates:** Use Yew function components and hooks (`use_state`, `use_mut_ref`) for state management.

## Project-Specific Patterns
- **Share Codes:** Format is `CL|DP-WORD##` (e.g., `CL-PANTS42`). Used for deterministic seeds and replaying runs. See `GameState` in `src/game/state.rs` and regex in `src/components/ui.rs`.
- **Modes:** `Classic` and `Deep End` are set via share code prefix and affect available encounters.
- **Boot Overlay:** Loading sequence is managed in `src/app.rs` with phased state.
- **Accessibility:** Keyboard navigation and toggles for high contrast/reduced motion are present in UI components.
- **Assets:** All images and data are loaded from the `assets/` directory. Preloading logic in `src/app.rs`.

## Conventions
- **Rust/Yew idioms:** Use function components, props, and hooks for UI/state. Avoid class components.
- **Data-driven:** Encounters and choices are defined in JSON, not hardcoded.
- **No network calls except asset fetches.**
- **No global mutable state outside Yew hooks.**

## Example: Adding an Encounter
1. Add to `assets/data/game.json`.
2. Update models in `src/game/data.rs` if needed.
3. UI will reflect new encounters automatically if data shape matches.

---
For questions or unclear conventions, review `PROJECT_PLAN.md` and `README.md` for rationale and gameplay details.
