# Journey System Overview

**Purpose:** Explain the moving parts of a Dystrail run: how configs are layered, how determinism is enforced, and which data packs drive behavior.

## Core architecture
- One **JourneyController** simulates every day from mile 0 to the boss gate (default route length `2100.0` in `boss::ROUTE_LEN_MILES`).
- Each day emits a `DayRecord` (kind: `Travel`, `Partial`, `NonTravel`) with miles already partial-adjusted and tagged.
- Game mode → policy family mapping:
  - **Classic** family (GameMode::Classic) — baseline Oregon Trail pacing.
  - **Deep** family (GameMode::Deep) — same pacing bands, higher variance/quirks.
- Strategy overlay: Balanced, Aggressive, Conservative, ResourceManager. The overlay only changes numbers; code paths stay uniform.

## Determinism & RNG streams
- Seeds derive from HMAC-SHA256(master_seed, domain_tag) → 64-bit seeds (see `journey::RngBundle`).
- Independent streams: `rng_travel`, `rng_breakdown`, `rng_crossing`, `rng_encounter`.
- **Atomic crossings:** exactly one RNG draw per crossing event regardless of outcome branch, preventing seed drift.
- Replay guarantee: same seed + same data packs = identical simulation trace (used by tester suites).

## Data packs and source of truth
All policy and tuning data live in `dystrail-web/static/assets/data/` and are bundled into the shipped WASM build.

- Journey families: `journey/classic.json`, `journey/deep.json`.
- Strategy overlays: `journey/overlays/*.json` (merged on top of family data per field).
- Supporting packs:
  - `boss.json` (distance required, weights, rounds, min/max chance, biases)
  - `crossings.json`, `camp.json`, `exec_orders.json`, `endgame.json`
  - `pacing.json`, `personas.json`, `result.json`, `store.json`, `vehicle.json`, `weather.json`, `game.json`

**Effective config** = family ⊕ overlay. Examples (Classic + Balanced overlay):
- Victory miles: 2100 → **2400** (overlay).
- Partial ratio: 0.50 → **0.51**.
- Travel base mpd: 14.6 → **15.6**; clamp remains 10.0–22.0.
- Wear base/fatigue: 0.07 / 0.08 → **0.073 / 0.245**.
- Breakdown base/beta: 0.01 / 0.10 → **0.015 / 0.185**.
- Crossing probs: pass/detour/terminal 0.72 / 0.16 / 0.12 → **0.66 / 0.24 / 0.10**; bribe/permit tuning updated accordingly.
