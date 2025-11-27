# Dystrail Web UX & Color Palette Specification

_Authoritative UX + Visual Spec for Non‑Image Assets_

## 1. Overview

This document defines the complete **UI/UX behavior**, **non-image asset standards**, and **color palette rules** for Dystrail’s SNES‑lite interface. All UX structures are derived from the verified project plan, gameplay loop, and all loaded game data sources.

Aesthetic goals:

- SNES-lite warm sepia palette
- No grid/pixel lattice lines
- Implied pixelation via clustered shading & dithering
- 4K clarity
- Consistent top-left lighting
- WCAG‑AA compliant

### 1.1 Data Schemas (authoritative summary)

- **Encounters (`game.json`)**: id, name, desc, weight ≥0, regions[], modes[], choices[] (label + effects bundle for stats/log). Ids unique; at least one choice; regions/modes non-empty.
- **Store (`store.json`)**: categories with id/name/items; items have id, name, desc, price_cents, unique flag, max_qty, grants (stats/inventory), tags. Unique implies max_qty=1; tags must match logic.
- **Weather (`weather.json`)**: effects per type (supplies/sanity/pants deltas, encounter delta, travel mult), mitigation tags, regional weights, exec-order encounter mods, limits (streak cap, encounter cap, pants floor/ceiling). Weights per region must sum >0.
- **Boss (`boss.json`)**: distance_required, rounds, per-round sanity/pants, base/min/max win chance, stat weights (supplies/sanity/cred/allies/pants), deep/classic bias tweaks. Min chance ≤ max; all probabilities 0–1.
- **Result (`result.json`)**: pants score threshold/penalty, persona multiplier rules, ending priority (pants > sanity > collapse > boss_loss > victory), headline keys, share length limits.
- **Executive Orders**: event-driven; canonical set = Shutdown, Travel Ban Lite, Book Panic, Tariff Tsunami, DoE Eliminated, War Dept Reorg, NatGuard Deployment (no timeline file).
- **Crossings (`crossings.json`)**: types (checkpoint, bridge_out) with detour cost (days/supplies/pants), bribe (cost, success, fail penalties), permit (cred gain); seasonal thresholds; weather/EO global mods; money rules (budget cannot go negative in v1 OT parity; set allow_negative_budget=false).
- **Camp (`camp.json`)**: actions define day cost, stat deltas, supplies burned, cooldowns, forage outcome weights; therapy/repair fully defined (see Camp Panel).
- **Pacing & Diet (`pacing.json`)**: pace modes (steady/heated/blitz) affecting distance, sanity, pants, encounter delta; diet modes (quiet/mixed/doomscroll) affecting receipt odds, sanity, pants; global limits (encounter base 0.27, distance base, pants floor/ceiling).
- **Personas (`personas.json`)**: name, desc, starting stats, score multiplier, mods (receipt find, discounts, EO mitigation, pants relief thresholds).

### 1.2 State & Session Flow

- Phase order: Boot (load data) → Persona Select → Outfitting Store → Menu hub → Travel → Camp (optional) → Encounter (if triggered) → Boss (after distance threshold) → Result → Restart/New Run. Menu fans out to Travel, Camp, Save/Load, Settings; otherwise phases advance forward.
- Seed/RNG: share-code derives deterministic seed; RNG reconstructed on load (saves store state only). Weather, encounters, crossings, travel outcomes, boss outcomes all seeded.
- Day cadence: Start-of-day rolls weather, EO state, persona passives. Travel applies pace/diet/weather/EO effects and may branch to crossing/encounter/none. End-of-day clamps stats (pants ceiling), checks breakdowns/boss unlock, and shows post-travel state before next input.
- Panel transitions: all panels switch instantly (no transition animations). Escape only closes modals/drawers; it never changes panels. Back exists only where explicitly shown (crossing “0=Back”, submenu back buttons, save/settings close).
- EO system is event-driven (no day-by-day timeline); only currently active EO(s) render in UI.

---

# 2. Global Visual Language

### 2.1 Style Foundations

- 16px base grid for all spacing and layout.
- Pixel art **implied**, never explicit.
- Rounded 4px “pseudo-pixel” corners on panels.
- 1px outer border + 1px inner shadow on all UI panels.
- All UI assets must remain crisp at 4K scaling.

### 2.2 Panel Layout Structure

All main panels follow the OT-style template:

```
┌────────────────────────────────────────────┐
│ Header: Persona • Stats • Weather • EOs    │
├────────────────────────────────────────────┤
│ Body: Travel / Encounter / Store / Etc     │
├────────────────────────────────────────────┤
│ Footer: Seed Bar • Navigation Controls     │
└────────────────────────────────────────────┘
```

Sections follow left→right and top→bottom logical ordering for accessibility.

---

# 3. Color Palette Specification (Non‑Image Assets)

Palette is authoritative and locked (based on Missing Assets Inventory).

### 3.1 Standard Theme (OT‑Inspired Warm Frontier Palette)

| Role                    | HEX     | Notes                                      |
| ----------------------- | ------- | ------------------------------------------ |
| Background              | #0A0907 | Deep CRT brown-black from OT title screens |
| Panel Background        | #3A2918 | Saddle-brown UI backdrop seen in menus     |
| Panel Border            | #E4C07A | Parchment-gold border highlight            |
| Primary Text            | #F2D7A0 | Warm parchment tone from OT dialogue       |
| Dim Text                | #BFA782 | Muted prairie clay                         |
| Bright Text             | #FFF2C9 | High-luminance lantern parchment           |
| Accent Primary          | #F7E39B | Frontier gold highlight                    |
| Accent Secondary        | #D29A43 | Warm amber                                 |
| Button Background       | #54381F | Dark leather brown                         |
| Button Border           | #E4C07A | Same gold as panels                        |
| Shadow                  | #140E07 | Deep smokey shadow                         |
| Prairie Green (UI Only) | #6F8B45 | OT field‑tone green                        |
| Trail Dust Grey         | #8A7E6A | Dust/silt grey from UI elements            |

### 3.2 High‑Contrast Theme (OT‑Compatible)

| Role             | HEX     | Notes                      |
| ---------------- | ------- | -------------------------- |
| Background       | #0A0F14 | Deep blue‑black (CRT safe) |
| Panel Background | #162128 | Slate gunmetal             |
| Panel Border     | #F2F2F2 | Maximum contrast           |
| Primary Text     | #F2F2F2 | High readability           |
| Dim Text         | #D0D6DD | Steel grey                 |
| Bright Text      | #FFFFFF | Pure white                 |
| Accent Primary   | #00E0CE | Readable turquoise accent  |
| Accent Secondary | #00A896 | Teal secondary             |

### 3.3 Contrast Requirements

- Text must maintain ≥ 4.5:1 contrast on both standard and high‑contrast themes.
- Icons ≥ 3:1 contrast minimum.
- Hover/focus states must use increased luminance or border contrast, never color‑only changes.

---

# 4. Core UI Components

## 4.1 Header Row (Persona + Stats + Conditions)

Data sources:

- `personas.json` (starting stats & modifiers)
- `weather.json` (effects, mitigation)
- Exec order enum: Shutdown, TravelBanLite, BookPanic, TariffTsunami, DoEEliminated, WarDeptReorg, NatGuardDeployment
- Pants limits from `pacing.json` & `result.json`

### Layout & Behavior

- **Left:** 64×64 persona portrait.
- **Right:** Horizontal stat bar for: Supplies • HP • Sanity • Credibility • Morale • Allies • Pants.
- All stats use 16×16 icons with #F4E4C1 foreground and #1A1000 outline.
- Pants meter glows when ≥70 and pulses when ≥90.
- Weather + EO icons positioned right of stat bar:
  - Weather icon 24×24
  - EO icons 20×20 (scroll row if multiple active)

### Updating Rules

- Stats update on: encounters, travel, camp, EOs, breaks.
- Weather mitigation reduces icon brightness 25%.
- EO icons appear only while active.

---

# 5. Buttons, Inputs & Interactives

## 5.1 Standard Buttons

- BG: #4A3728
- Border: #D4A574
- Text: #F4E4C1
- Hover: lighten BG by ~5%
- Active: inset shadow, darken inner edge to #1A1000
- Focus: 1px bright (#F4E4C1) focus ring
- Disabled: BG darkened to #2D1B00`, text `#B8956A

## 5.2 Input Fields (Seed, text inputs)

- BG: #000000
- Border: #D4A574
- Text: #F4E4C1
- Focus state: glowing border +2px internal highlight
- Validation: error state uses #8a2c2c underline + accessible text

---

# 6. Panel-by-Panel UX Specification

Phase transitions:

- Boot → Persona Select → Outfitting Store → Menu (hub) → Travel (one day) → optional Camp → Encounter (if triggered) → Boss (after threshold) → Result → Restart/New Run.
- Menu fans out to Travel, Camp, Save/Load, Settings; modals/drawers trap focus and restore to opener on close.

# 6.1 Boot & Loading Panel

Data source: gameplay loop summary.

### Requirements

- Preload all JSON datasets: `game.json` (encounters), `store.json`, `personas.json`, `weather.json`, `pacing.json`, `crossings.json`, `camp.json`, `result.json`, `vehicle.json`, `boss.json`, `endgame.json`.
- Loading bar uses:
  - BG: #2D1B00
  - Fill: #D4A574
- “Press Any Key to Begin” cycles brightness (no motion blur).

---

# 6.2 Persona Selection Panel

Data source: `personas.json`

### Tile Structure

- 64×64 persona portrait
- Name (primary text)
- Description (dim text)
- Starting stat preview (mini-icons)
- Hover/focus: panel border switches to #F4E4C1
- Selected: border pulses subtly (40% brightness oscillation)

### Behavior

Selecting a persona:

- Applies starting stats + mods
- Updates score multiplier
- Enables transition to Store / Mode screen

---

# 6.3 Outfitting Store UX

Data source: `store.json`

### Layout

- Category tabs across top: Fuel/Food, Vehicle, PPE, Docs.
- Item grid uses:
  - 32×32 icon
  - Name, price, description tooltip
  - Quantity +/- controls
- Budget displayed top-right in bright text.

### Behavior

- Unique (max_qty=1) items disable + after purchase.
- Buying applies `grants` immediately (supplies, spares, credibility).
- Lobbyist mod applies store_discount_pct to all prices.
- Budget cannot go negative (OT parity override). Persona store discount is the only discount; apply once, round half-up to cents. Prices shown as `$X.YY` (no thousands separators; negatives as `-$X.YY`). +/– respects max_qty/unique. Categories and items show numeric shortcuts: categories 1–4 (Fuel/Food, Vehicle, PPE, Docs), items 1–N within category; navigation is category number then item number. Store completes to the Menu hub; categories switch via tabs or numeric shortcuts. On insufficient funds, show “INSUFFICIENT FUNDS” and block the purchase.

---

# 6.4 Share Code + Mode Select Panel

Data source: project plan.

### Requirements

- Seed input supports regex: `^(CL|DP)-[A-Z0-9]+\d{2}$`
- “Classic” and “Deep End” mode buttons
  - DP uses subtle red hue while staying in palette rules
- “Randomize Seed” regenerates code
- Starting run locks persona + mode + seed

---

# 6.5 Travel Panel UX

Data sources: `pacing.json` (encounter_base=0.27, pace deltas), `weather.json` (enc_deltas), regions.

### Layout

- Region background (Heartland / RustBelt / Beltway)
- Header stats + conditions
- Middle:
  - Day counter
  - Distance marker
  - Pace selector
  - Diet selector

### Behavior

Encounter chance = `0.27 + pace.encounter_delta + weather.enc_delta` (clamped by pacing/weather limits).

Travel costs:

- Supplies, sanity, pants (pace/diet/weather)
- EO global modifiers (Shutdown, TariffTsunami, etc.)

Interaction:

- Single primary action: “Travel Next Leg.”
- Hover/focus on pace/diet shows predicted daily deltas before committing.
- Numeric shortcuts: pace/diet options numbered in visual order; Travel button is activated by Enter/Space only.

Travel log:

- aria-live, dim text (#B8956A), auto-scroll but never traps focus.

---

# 6.6 Encounter Panel

Data source: `game.json` events.

### Structure

- Large illustration
- Title + descriptive text
- 2–3 choice buttons, numbered 1–3 in visual order (1 = Continue if only one)
- Effects preview on hover:
  - hp, sanity, supplies, credibility, morale, pants, allies
  - receipts gained or burned
- Deep-only events hidden unless mode=DP.

### Behavior

- Effects applied atomically.
- Log entry added after player choice.
- Panel uses a 16px padding frame.
- Focus order: heading → choice 1 → choice 2 → choice 3 → footer/back (if present).
- Tooltips/previews show dim description + bulleted stat/time/risk deltas on hover/focus/first tap; second tap activates on touch.

---

# 6.7 Crossing Panel (Checkpoint & Bridge Out)

Data source: `crossings.json`.

### UI

Each crossing shows:

- Illustration
- Three option buttons: Detour, Bribe, Permit
- Inline cost display:
  - Detour: days/supplies/pants
  - Bribe: cost + EO-modified fail chance
  - Permit: credibility gain + Press Pass requirement

### Rules

- EO Shutdown modifies bribe success chance.
- Press Pass (Docs category) acts as permit.
- Options are fixed: Detour=1, Bribe=2, Permit=3, Back=0. Focus order: heading → detour → bribe → permit → back. Tooltips show costs, time, and risk deltas on hover/focus/first tap.
- Budget cannot go negative for bribes; block with “INSUFFICIENT FUNDS” when short.

---

# 6.8 Camp Panel

Data source: `camp.json`.

### Layout

Action tiles reflect available data:

- **Rest**: +2 sanity, +1 hp, –1 supplies, +1 day.
- **Therapy**: +2 sanity, burns 1 receipt, +1 day, cooldown 3 days, no supplies cost.
- **Forage**: Outcome weights: supplies 50%, none 30%, receipt 20% (receipt chance boosted by persona up to +25% cap); no explicit day cost; supply gains use the baseline + region multipliers.
- **Repair**: Hack repair costs 3 supplies, 1 credibility, +1 day; spare repair consumes 1 supply and 1 matching spare (no credibility/time cost).

Each tile shows tooltip with exact numbers where data exists.

- Numeric shortcuts: Rest=1, Therapy=2, Forage=3, Repair=4. Focus order: heading → 1 → 2 → 3 → 4.

---

# 6.9 Vehicle Panel

Data source: `vehicle.json`.

### Elements

- Vehicle status illustration
- Breakdown type indicator: tire, battery, alternator, pump
- Repair options:
  - Use spare
  - Hack repair (supplies, credibility, day cost)
  - Mechanic hook if enabled

### Rules

- Breakdown chance scales with pace + weather.
- Spare consumption automatically updates store inventory state.

---

# 6.10 Filibuster Boss Panel

Data source: project plan.

### Phase UX

**Phase 1 — Cloture**

- 3 action buttons each round: Hold • Present Receipts • Call Allies
- Credibility ≥10 auto-passes

**Phase 2 — Points of Order**

- Four role-check nodes (Researcher, Organizer, Healer, Scout)
- Each displays pass/fail indicator, based on weighted stat checks.

**Phase 3 — Amendment Flood**

- Card-draw UI with counters (receipt/ally/clarity)
- Survive M rounds to advance
- Pants meter always visible

---

# 6.11 Result Screen UX

Data sources: `result.json` (score clamp + ending priority).

### Layout

- Big headline variant (victory, pants fail, sanity fail, collapse, boss loss)
- 200×150 illustration
- Final runtime stats table (days, encounters, receipts, allies, pants)
- Score calculation display with persona multiplier and pants penalties
- Seed (copyable)
- “Share Run” (JSON or formatted text)

### Ending Priority Rules

Order:

1. pants
2. sanity
3. collapse
4. boss_loss
5. victory

Score clamped to [0, 999999].

---

# 7. Iconography & Non-Image Asset Rules

### 7.0 Sprite Identifiers (from ASSETS.md)

- Stats: stat_supplies, stat_hp, stat_sanity, stat_credibility, stat_morale, stat_allies, stat_pants
- Weather: weather_clear, weather_storm, weather_heatwave, weather_coldsnap, weather_smoke
- Executive Orders: eo_shutdown, eo_travel_ban, eo_book_panic, eo_tariff_tsunami, eo_doe_eliminated, eo_war_dept, eo_natguard
- Pace/Diet: pace_steady, pace_heated, pace_blitz, diet_quiet, diet_mixed, diet_doom
- Camp: camp_rest, camp_therapy, camp_forage, camp_repair
- Store: item_rations, item_water, item_spare_tire, item_battery, item_alternator, item_fuel_pump, item_masks, item_coats, item_ponchos, item_press_pass, item_legal_fund
- Personas: portrait_journalist, portrait_organizer, portrait_whistleblower, portrait_lobbyist, portrait_staffer, portrait_satirist
- Encounters: encounter_raw_milk, encounter_tariff_whiplash (minimum guaranteed)
- Regions: region_heartland, region_rustbelt, region_beltway
- Vehicle: vehicle_ok, vehicle_tire_blowout, vehicle_battery_dead, vehicle_alternator_failure, vehicle_fuel_pump_failure
- Result: result_victory, result_boss_loss, result_pants, result_sanity, result_collapse

## 7.1 Stat Icons (16×16)

- Foreground: #F4E4C1
- Outline: #1A1000
- Includes: supplies, hp, sanity, credibility, morale, allies, pants.

## 7.2 Weather Icons (24×24)

- Primary: #D4A574
- Highlights: #F4E4C1
- Variants: Clear, Storm, HeatWave, ColdSnap, Smoke.

## 7.3 Executive Order Icons (20×20)

- Monochrome #D4A574
- Must visually differentiate: Shutdown, Travel Ban Lite, Book Panic, Tariff Tsunami, DoE Eliminated, War Dept Reorg, NatGuard Deployment.
- Icons display only when EO is active; tooltip shows name + short summary. EO system is event-driven (no day-by-day timeline data).
- Tooltip summaries:
  - Shutdown — Reduced operations → bribes unstable.
  - Travel Ban Lite — Movement restricted → supplies drain.
  - Book Panic — Cultural unrest → sanity down.
  - Tariff Tsunami — Trade disruptions → supplies cost more.
  - DoE Eliminated — Education void → flavor only.
  - War Dept Reorg — Military reshuffling → encounter rate shifts.
  - NatGuard Deployment — Patrols increased → encounters up.
- Sprite map (1:1): Shutdown→eo_shutdown, Travel Ban Lite→eo_travel_ban, Book Panic→eo_book_panic, Tariff Tsunami→eo_tariff_tsunami, DoE Eliminated→eo_doe_eliminated, War Dept Reorg→eo_war_dept, NatGuard Deployment→eo_natguard.

## 7.4 Pace & Diet Icons (24×24)

- Pace: Steady, Heated, Blitz
- Diet: Quiet, Mixed, Doomscroll
- Shape-based differences required for accessibility.

## 7.5 Camp Action Icons (48×48)

- Rest, Therapy, Forage, Repair
- Must remain readable at 1× scale.

## 7.6 Placeholder Visuals

- If a sprite is missing, show a 2-character text badge centered in a parchment box (panel BG + border).
- Examples: Encounter=EV, Weather=WX, EO=EO, Pace=PC, Diet=DT, Store items=two-letter code (RA, WT, etc.).

---

# 8. Accessibility Requirements (WCAG 2.2 AA)

- Full keyboard operability (Tab, Shift+Tab, Enter, Space, Arrow keys).
- Visible custom focus rings on all interactive components.
- aria-live for travel logs.
- aria-modal + focus trap for overlay panels.
- High-contrast theme toggle must persist in localStorage.
- RTL layout support for Arabic (logical CSS: margin-inline-start, etc.).
- No hover-only information; all must be accessible via focus.

---

# 9. Engineering Integration Notes

- All palette values encoded as CSS variables; components must not hardcode colors. High-contrast mode swaps the entire variable set.
- Theme variable inventory (conceptual): colors (bg, panel bg, panel border, text primary/dim/bright, accent primary/secondary, button bg/border, shadow, focus ring); spacing (grid base 16px, 8/16/24px steps, panel padding 16–24px, button padding 12–20px); borders (1px width, 4px radius); focus (2–3px outline, 2px offset); motion (fast 0.15–0.2s, slow pulse 1.5–2s).
- JSON-driven encounters, store, camp, EO data map directly to reusable components; update `camp.json` to reflect Rest/Therapy/Forage/Repair values above.
- Sprite atlas (when built) will be image-only; all UI elements here are CSS/DOM. Use placeholder badges when sprites missing.
- Responsive scaling: maintain base 16px grid, upscale proportionally. Touch-friendly: minimum interactive area 44×44px.
- i18n namespace map: menu._, persona._, store._, travel._, camp._, encounter._, weather._, eo._, result._, seed._, error._, log._, save.\*.
- Error/empty/loading text: Loading panel “LOADING…”; data error “DATA FAILED TO LOAD — TRY AGAIN”; empty lists show “NONE”; invalid seed “INVALID CODE FORMAT” + “VALID EXAMPLE: CL-PANTS42”; insufficient funds “INSUFFICIENT FUNDS”; save failure “COULD NOT SAVE GAME”; load failure “INVALID SAVE FILE”.
- Save/Load drawer: instant open/close; actions with numeric shortcuts (1 Save, 2 Load, 3 Export, 4 Import, 0 Back); strings under save.\*; Esc closes and returns focus to opener.
- EO canon: code enum + assets must match the 7 canonical EOs (Shutdown, Travel Ban Lite, Book Panic, Tariff Tsunami, DoE Eliminated, War Dept Reorg, NatGuard Deployment); drop or ignore any other EO sprites.
- Forage receipt cap: implement the +25% persona bonus cap on receipt chance alongside the 50/30/20 weights.
- Budget enforcement: set allow_negative_budget=false in data, and block overspend in store/crossings with “INSUFFICIENT FUNDS.”
- Sprite coverage: provide sprites or fallback badges for `encounter_raw_milk`, `encounter_tariff_whiplash`, and all canonical EO IDs.
- Keyboard shortcuts: render and bind numeric shortcuts for encounters (1–3), crossings (1/2/3, 0 back), camp (1–4), pace/diet (in order), and store (category number then item number).
- Placeholder badges: implement the 2-letter fallback helper in components for any missing sprite keys.
- i18n coverage: add locale keys for EO tooltips, camp tooltips, loading/error/empty states, budget/seed/save/load errors, and badge/placeholder labels under the namespaces above.

---

# 10. Definition of Done (Web UX)

- Pixel-perfect adherence to palette & edge rules
- All panels implement the OT-style layout
- Stats & conditions always visible (header)
- Seed bar always visible (footer)
- High-contrast theme fully functional
- Keyboard walkthrough possible end-to-end
- RTL support verified
- All text comes from i18n JSONs
- No visual gridlines used anywhere
- 4K rendering validated

---

_This spec is now the authoritative reference for all Dystrail UI/UX implementation work._
