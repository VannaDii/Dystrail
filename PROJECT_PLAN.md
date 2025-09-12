# Dystrail — Project Plan (for Codex)

**Date:** 2025-09-12
**Target:** Playable web demo with satire loop, share codes, and screenshot-worthy results.

## 1) Scope Summary
Retro parody survival game. Travel to DC, survive satirical hazards, manage stats, face the Filibuster Boss, and screenshot your ending with a short share code.

### Goals
- Deterministic runs via **Share Codes** (`CL|DP` + WORD + 2 digits)
- **Mode Select**: Classic vs The Deep End
- **Data-driven encounters** from `assets/data/game.json`
- A11y (keyboard nav, high contrast, reduced motion)
- Viral visuals: strong palette, readable pixel UI, OG cards

### Non-Goals (v1)
- Network services, accounts, or backend
- Sophisticated animation or physics
- Multiplayer or real-time systems

## 2) Tech & Architecture
- **Stack:** Rust + Yew → WASM; static hosting
- **Modules:**
  - `app` — overlay, boot, routing to game panels
  - `components/ui` — Stats, Share Code Bar, Mode Select, Result Screen
  - `game/state` — model (stats, mode, seed, progress)
  - `game/data` — fetch and parse `assets/data/game.json`
  - **(Next)** `game/encounters`, `game/exec_orders`, `game/boss`

- **Assets:** `assets/gfx/*.png`, `favicon.ico`, OG card

## 3) Game Model
### Stats
- `Supplies`, `HP`, `Sanity`, `Credibility`, `Morale`, `Allies`
- `PantsMeter` (0..100): at 100 → **National Pants Emergency** (fail)

### Modes
- `Classic` (`CL`): broad satire, lighter set
- `Deep End` (`DP`): edgier/illicit events

### Share Codes (short format)
- **Format:** `<MODE>-<WORD><NN>` (e.g., `CL-PANTS42`, `DP-GATOR97`)
- **WORD:** curated 512-word list (caps, ≤7 letters), includes **ORANGE, CHEETO, MANGO**
- **NN:** 00..99 suffix
- **Deterministic mapping:** seed ↔ code (spec below)
- **UI:** Share Code input above Mode Select; prefilled, editable; applying a code sets mode and starts run

### RNG & Seeds (Spec)
- Use `rand_chacha` for deterministic rolls
- Seed encoder/decoder (from design draft):
  - `word_index` ∈ [0..511]
  - `nn` ∈ [0..99]
  - derive `u64` seed deterministically from `(mode, word, nn)` using a stable hash (e.g., FNV-1a) + mixing
- **Deliverables:**
  - `seed.rs` with:
    - `const WORD_LIST: [&'static str; 512]` (curated tokens)
    - `encode_friendly(is_deep: bool, seed: u64) -> String`
    - `decode_to_seed(code: &str) -> Option<(is_deep: bool, seed: u64)>`

## 4) Loop & Panels
1) **Boot Overlay**
   - Preload `assets/data/game.json`, `gfx/logo.png`, `gfx/spritesheet.png`
   - Progress bar → **Ready** → Start button

2) **Share Code + Mode Select**
   - Share Code (prefilled; paste to replay); validator: `^(CL|DP)-[A-Z0-9]+\d{2}$`
   - Buttons: **Classic**, **The Deep End** → new random code → start

3) **Travel Panel**
   - Show stats + day/region
   - “Travel Next Leg” → costs Supplies/Sanity, region progression, 35% encounter chance
   - Log list (aria-live)

4) **Encounter Panel**
   - Event card with sprite, description, 2–3 buttons
   - Outcomes adjust stats; may add/remove `Receipts` or `Allies`
   - PantsMeter bumps on stress
   - **Deep-only** flags honored by mode
   - JSON schema (data-driven):  
```json
{ "id": "string", "name": "string", "desc": "string",
  "weight": 5, "regions": ["Heartland","RustBelt","Beltway"],
  "modes": ["classic","deep_end"],
  "choices": [{ "label": "string", "effects": { "hp":0, "sanity":0, "credibility":0, "supplies":0, "morale":0, "allies":0, "pants":0, "add_receipt":"opt", "use_receipt":true, "log":"opt" } }]
}
```

5) **Executive Orders** (rotate every ~3–5 days)
- Government Shutdown, Travel Ban Lite, Gas-Stove Police, Book Panic, Deportation Sweep, Tariff Tsunami, DoE Eliminated, War Dept Reorg
- Implement as global modifiers that tick per day; show current EO in Stats panel

6) **Filibuster Boss**
- **Phase 1: Cloture** — reduce Pressure via **hold**, **present receipts**, **call allies**; Cred ≥ 10 auto-pass
- **Phase 2: Points of Order** — 4 role checks (Researcher/Organizer/Healer/Scout), survive N rounds
- **Phase 3: Amendment Flood** — draw counters (receipt/ally/clarity); survive M rounds
- PantsMeter can trigger fail at any phase

7) **Result Screen**
- Bold headline (e.g., **YOU PASSED CLOTURE!**, **NATIONAL PANTS EMERGENCY**)
- Stats table (days, encounters, receipts, allies, pants)
- Seed visible and copyable
- **Share Run** button (copy text/JSON; future: render to PNG)

## 5) Visuals & A11y
- Palette locked (`assets/gfx/palette.png`), SNES-lite style
- Sprites: 32×32 tiles in `assets/gfx/spritesheet.png`
- Event Card: framed dialog with pixel headline + buttons; large touch targets
- Accessibility: WCAG AA contrast, keyboard focus order, aria-live logs, progressbar roles, reduced motion honored

## 6) Sound (Phase 2)
- SFX: alert chime (encounter), success/fail stings, filibuster fanfare, pants-fail puff
- Mute toggle; default off; no autoplay

## 7) Data & Save
- Encounters: `assets/data/game.json` mixed with built-ins
- Save: localStorage (game state + seed + mode)
- Export/Import: JSON save

## 8) QA & Telemetry
- Deterministic seeds for bug repro
- Console shows seed + version
- No external telemetry (static site)

## 9) Milestones
- **M1:** Merge seed encoder/decoder; implement Encounter registry + EO modifiers; hook Travel + Encounter UI; JSON loading.
- **M2:** Filibuster Boss phases + Result Screen component.
- **M3:** Save/Load + Export/Import; SFX preload; polish + a11y pass.
- **M4:** Content expansion (≥20 encounters; deep-only set), backgrounds, end-screen exports.

## 10) Acceptance Criteria
- Seed `DP-ORANGE42` produces the same run across devices
- Mode filters correctly apply deep-only events
- PantsMeter fail triggers the Pants Emergency ending
- Boss phases are playable and end in a consistent Result Screen
- OG/Twitter cards unfurl correctly using `assets/gfx/social-card.png`
