# Data Packs & Modding Guide

Everything that shapes a run lives in JSON under `dystrail-web/static/assets/data/` and ships with the WASM build. Editing these files lets you reskin or satirize the game without touching Rust.

## Where things live
- `journey/classic.json`, `journey/deep.json`: base family configs (mpd ranges, partial ratio, wear/breakdown, crossings, guards).
- `journey/overlays/*.json`: strategy overlays that override family fields (Balanced, Aggressive, Conservative, ResourceManager).
- `boss.json`: distance gate, round count, stat weights, min/max chance, balanced bias (Classic bonus, Deep multiplier/bonus).
- `crossings.json`, `camp.json`, `exec_orders.json`, `endgame.json`: crossings odds/detours, camp actions, executive orders, endgame behavior.
- `pacing.json`, `weather.json`, `vehicle.json`: pace multipliers, weather impacts, vehicle wear/parts weights.
- `personas.json`, `store.json`, `result.json`, `game.json`: flavor, pricing, outcomes, and high-level game toggles.

## How to make a new variant
1. **Copy and edit JSON** in `static/assets/data/`. Change numbers, names, or odds to your liking.
2. **Run tests**: `just lint` or `cargo test --workspace --all-features --locked` to ensure acceptance guards still pass.
3. **Build the web client**: `just build-release` (or let the CI `build` job run). The JSON is bundled into `dystrail-web/dist`.
4. **Ship it**: host `dist` (GitHub Pages via included workflows) and share the Play link/download.

## Knobs to twist for satire
- **Crossings**: raise `terminal`, shrink `pass`, or make `detour_days.max` huge; tweak bribe bonuses/penalties.
- **Travel feel**: drop `mpd_base`, shrink `mpd_min`, or slash `partial_ratio` to 0.2 for a slog; invert for speed-runs.
- **Breakdowns**: spike `breakdown.base` and `beta`, or overweight a single part in `part_weights`.
- **Endgame**: set `wear_multiplier` to 0 to make finale trivial, or raise `health_floor` to punish.
- **Boss**: rename outcomes in `boss.json`, push `distance_required` up/down, or skew stat weights to reward pants hoarding.
- **Economy/Flavor**: rewrite `store.json` prices, `camp.json` actions, `personas.json` lines, `weather.json` names to match your satire.

## Notes
- Keep `guards` reasonable if you want CI to stay green; otherwise relax them knowing tests may fail.
- Determinism is preserved as long as you keep the same RNG structure—changing JSON won’t break replays for a given seed, it only changes the outcomes via new numbers.
