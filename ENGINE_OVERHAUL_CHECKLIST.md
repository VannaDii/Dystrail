# ENGINE_OVERHAUL_CHECKLIST.md
STATUS: AUTHORITATIVE IMPLEMENTATION CHECKLIST (derived from sacred specs)

This file is the implementation checklist to fully realize the following authoritative documents:
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md` (primary normative spec; implementation-binding)
- `ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md` (normative kernel pseudocode)
- `ENGINE_JOURNEY_CONTROLLER_DIFF.md` (delta/mapping vs current pipeline)

Parity target:
- Oregon Trail Deluxe (DOS v3.0, ’90s lineage) via `OTDeluxe90sPolicy`

Non-goals:
- No implementation in this checklist (documentation only).
- Satire is presentation-only and MUST NOT affect mechanics, RNG, or phase order.

How to use this checklist:
- Each checkbox is a concrete deliverable with acceptance criteria.
- “P0” items are blockers for spec compliance; do them first.
- Where the spec allows optional/future modules (e.g., climate-station weather), those are explicitly marked.
- When a requirement changes numeric behavior or RNG consumption, ensure deterministic replay and update baseline tests.

Legend:
- Priority: P0 (blocking), P1 (high), P2 (medium), P3 (nice-to-have)
- “Spec refs” point into the three sacred docs above.
- “Determinism notes” describe RNG stream constraints and replay implications.

Locked decisions already made (DO NOT re-litigate during implementation unless new Deluxe evidence appears):
- Oregon City is presentation-only (not a distinct node).
- `STORE_PRICE_MULT_PCT_BY_NODE` index-to-node mapping is treated as normative.
- MECC Appendix B numeric thresholds are treated as binding for Deluxe parity and live in policy (not hard-coded).
- Affliction probability curve defaults to the piecewise-linear `AFFLICTION_CURVE_PWL` described in the systems spec (policy-owned).
- Hunting carry cap uses `100 * alive_party_members` and “alive” means “not dead” (injuries still count).
- Ferry defaults:
  - `FERRY_MIN_DEPTH_FT = 2.5`
  - wait-days distribution is uniform `0..=6` (until extracted/fitted)
  - accident bucket is non-lethal by default (until Deluxe evidence proves ferry deaths)
- Snake River guide defaults:
  - `GUIDE_RISK_MULT = 0.20`
  - `GUIDE_LOSS_MULT = 0.50` (until extracted/fitted)
- River swiftness is modeled as continuous (no invented hard threshold gate unless extracted).
- Ford wet-goods defaults:
  - depth `2.5..=3.0` triggers `DRYING_COST_DAYS = 1` and does not cause permanent loss by default.
- Wagon capacity model defaults to per-item caps only (no global weight cap unless proven later).
- Weather generator remains Dystrail-regional (`weather.json`) for now, but must preserve MECC/Deluxe causal fan-out; snow accumulators remain present even if dormant.
- Arrival scoring defaults:
  - `SCORE_POINTS_PER_PERSON_BY_HEALTH`: Good=500, non-Good=0 until proven otherwise
- Death-imminent defaults:
  - `DEATH_IMMINENT_GRACE_DAYS = 3` and `reset_on_recovery_below_threshold` (until extracted/fitted)
- Occupation numeric defaults are accepted for now (policy-owned knobs):
  - `OCC_DOCTOR_FATALITY_MULT = 0.50`
  - `OCC_REPAIR_SUCCESS_MULT = 1.25`
  - `OCC_MOBILITY_FAILURE_MULT = 0.75`

Coverage index (quick “did we miss anything?” map):
- Systems spec MUST coverage (high-level index; exhaustive mapping is in Appendix A):
  - Satire/value-neutral kernel + i18n-only narrative + deterministic extra flavor: `GLOBAL-002`, `UI-002`
  - Policy overlays required + no piecemeal mixing: `GLOBAL-003`, `POLICY-001`, `POLICY-002`
  - Weather causal fan-out + snow hook: `WEATHER-002`, `WEATHER-003`
  - Health thresholds and arithmetic policy-owned: `HEALTH-001`, `HEALTH-007`
  - Route variants skip nodes + prompt nodes: `TRAIL-001`, `TRAIL-002`, `TRAIL-004`, `STORE-001`, `AUDIT-016`
  - The Dalles hard gate: `TRAIL-003`, `UI-001`
  - Event selection explainability: `EVENT-002`, `ENCOUNTER-003`
  - Store units/caps parity: `STORE-002`, `STORE-004`
  - Strain derived scalar (deterministic, hidden): `HEALTH-004`
  - Affliction odds driver explicit per policy: `POLICY-004`, `HEALTH-002`
  - Distance “no miles leak” rule: `TRAVEL-003`, `TRAVEL-009`
  - Rest/Trade/Hunt day-atomic: `REST-001`, `TRADE-001`, `HUNT-001`, `RECORD-001`
  - Phase ownership boundaries enforced: `ARCH-004B`, `TEST-001`
- Kernel pseudocode MUST coverage:
  - Mechanical event kinds + satire contract: `GLOBAL-002`, `EVENT-001`, `UI-002`
  - Mile-marker `0` sentinel handling: `TRAIL-001`, `TRAIL-002`
  - Crossing outcome families: `CROSSING-002`
  - The Dalles gate: `TRAIL-003`
- Journey diff mandatory deltas coverage:
  - Weather-first ordering: `WEATHER-001`, `ARCH-002`
  - Explicit SuppliesBurnTick: `SUPPLIES-001`
  - Health scalar + affliction curve: `HEALTH-001`, `HEALTH-002`
  - Navigation hard-stops + multi-day delays: `TRAVEL-002`, `TRAVEL-009`
  - Intent-based non-travel days: `INTENT-001`, `REST-001`, `TRADE-001`, `HUNT-001`
  - Encounter chance single-source + phase-scoped RNG: `ENCOUNTER-001`, `RNG-001..RNG-006`

---

## Implementation Order (efficient sequence; follow this)

Use this ordering when turning the checklist into PRs. Within each step, do **P0** items before **P1+**.

1) **Lock decisions (do first)**
   - Answer the questions in **Section 19 (Open Questions / Unresolvable Contradictions)** so architecture doesn’t churn mid-refactor.

2) **Scaffolding: policy + RNG + events + phase boundaries**
   - Mechanical overlay system + gating: `GLOBAL-003`, `POLICY-001..POLICY-006`
   - RNG domain expansion + phase guards: `GLOBAL-001`, `RNG-001..RNG-006`
   - Event bus + explainability: `EVENT-001..EVENT-003`
   - Phase ownership enforcement (structural): `ARCH-004B`, `TEST-001`

3) **Kernel entrypoints and orchestration cutover**
   - Kernel becomes the only day orchestrator: `ARCH-001`, `ARCH-002`, `ARCH-003`
   - Cut over existing orchestrators: `ARCH-005`, `ARCH-006`

4) **State model + migrations (foundation for all ticks)**
   - OTDeluxe core state + calendar + party + oxen: `STATE-001..STATE-004`, `TIME-001`, `PACE-001`, `RATIONS-001`, `OCC-001`
   - Save version + migrations: `MIGRATION-001`

5) **TrailGraph + route variants + store-node schedule (unblocks gates, stores, victory)**
   - Mile markers + sentinel handling: `TRAIL-001`, `TRAIL-002`
   - Branch prompts + Dalles gate: `TRAIL-003`, `TRAIL-004`, `UI-001`
   - Store-node semantics and multiplier schedule: `STORE-001`

6) **Implement daily ticks in phase order (root causes → gates → travel)**
   - Weather: `WEATHER-001..WEATHER-006`
   - Supplies: `SUPPLIES-001..SUPPLIES-004`
   - Health: `HEALTH-001..HEALTH-010` (includes pre-travel checks migration)
   - Boss gate: `BOSS-001` (Dystrail-only; policy-gated)
   - Forced waits: `WAIT-001` (ferry queues, drying days)
   - Intent resolution: `INTENT-001`, `REST-001`, `TRADE-001`, `HUNT-001`
   - Mobility + travel blocks: `TRAVEL-005`, `TRAVEL-006`, `TRAVEL-004`
   - EncounterTick (single-source chance + selection): `ENCOUNTER-001..ENCOUNTER-004`
   - ComputeMilesToday + TravelWearTick: `TRAVEL-001..TRAVEL-003`, `TRAVEL-007..TRAVEL-010`
   - EndgameTick (bounded): `ENDGAME-001`
   - CrossingTick (stop-and-choose in parity mode): `CROSSING-001..CROSSING-003`
   - RandomEventTick (non-navigation events): `ENCOUNTER-005`
   - RecordDay + TerminalChecks + EndOfDay: `RECORD-001`

7) **Economy/store (interactive purchasing + caps)**
   - Store caps + purchase units + money field alignment: `STORE-002..STORE-005`

8) **Scoring and endings**
   - Deluxe scoring: `SCORE-001`, `SCORE-002`

9) **UI integration (gates + rendering)**
   - UI gate states + event-driven rendering + satire/i18n constraints: `UI-001..UI-002`, plus any UI items referenced elsewhere.

10) **Determinism, tests, and lock-down harnesses**
   - Determinism tests + digests: `TEST-001..TEST-003`
   - EXE extraction + empirical fit harness (optional but recommended for “unknowns”): `LOCK-001`, `LOCK-002`
   - Run the preflight audit: `AUDIT-001..AUDIT-016`

---

## Appendix A) Spec MUST Coverage Map (exhaustive)

This section is a “nothing fell through the cracks” map: every `MUST` in the sacred specs points to one or more checklist IDs.

### `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md`

- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:46` policy-configure Dystrail-only translation layers → `POLICY-002`, `HEALTH-004`, `POLICY-004`, `POLICY-005`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:48` divergences must be explicit overlays (no piecemeal mixing) → `GLOBAL-003`, `POLICY-001`, `POLICY-006`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:95` kernel must remain value-neutral/mechanical → `GLOBAL-002`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:99` narrative text must be i18n-keyed → `GLOBAL-002`, `UI-002`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:100` satire must not change RNG/hidden modifiers/timing → `GLOBAL-002`, `AUDIT-011`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:101` optional satire flavor must be deterministic (no RNG) → `GLOBAL-002`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:118` wrapper may rename/narrate but must preserve event kind → `GLOBAL-002`, `EVENT-001`, `EVENT-003`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:207` satire rendering must not change RNG/phase order/numerics → `GLOBAL-002`, `AUDIT-011`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:212` unknown Deluxe numeric still must be explicit policy parameter → `POLICY-002`, `POLICY-005`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:307` unspecified mappings must be policy parameters (not hard-coded) → `POLICY-002`, `POLICY-005`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:318` DystrailLegacyPolicy must remain deterministic and fully specified → `GLOBAL-001`, `POLICY-001`, `POLICY-006`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:380` must carry `snow_depth` + slowdown hook → `WEATHER-003`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:389` must preserve Weather → fan-out causal chain → `WEATHER-002`, `WEATHER-001`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:423` health thresholds must live in policy/config → `HEALTH-001`, `POLICY-002`, `HEALTH-007`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:707` Sublette Cutoff must skip Fort Bridger node effects → `TRAIL-002`, `TRAIL-004`, `STORE-001`, `AUDIT-016`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:724` Dalles shortcut must skip Fort Walla Walla node effects → `TRAIL-002`, `TRAIL-004`, `STORE-001`, `AUDIT-016`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:743` combined shortcuts must skip both nodes → `TRAIL-002`, `TRAIL-004`, `STORE-001`, `AUDIT-016`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:836` optional flavor rendering must be deterministic by stable inputs → `GLOBAL-002`, `UI-002`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:852` The Dalles gate must block travel beyond node 16 until resolved → `TRAIL-003`, `UI-001`, `AUDIT-008`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:854` rafting/Barlow options must be deterministic, day-advancing subflows → `TRAIL-003`, `RECORD-001`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:855` policy must define rafting/Barlow costs/time/outcomes → `TRAIL-003`, `POLICY-005`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:882` event selection must produce explainable telemetry → `EVENT-002`, `ENCOUNTER-003`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:958` store purchase units and caps must match policy tables → `STORE-001`, `STORE-002`, `STORE-004`, `AUDIT-015`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1006` per-person points by health label must be an explicit policy parameter → `SCORE-001`, `POLICY-002`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1075` if future Deluxe capture contradicts score tiers, policy must be updated (kernel stays stable) → `LOCK-001`, `LOCK-002`, `SCORE-001`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1103` MECC health-scalar role must be replicated via derived `general_strain` computed daily → `HEALTH-004`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1107` `general_strain` must be deterministic → `GLOBAL-001`, `HEALTH-004`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1108` `general_strain` must not be player-visible → `HEALTH-004`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1158` affliction odds driver must be explicit per policy → `POLICY-004`, `HEALTH-002`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1227` OTDeluxe must enforce `distance_today_raw == distance_today` (no miles leak) → `TRAVEL-003`, `TRAVEL-009`, `AUDIT-005`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1261` rest/trade/hunt must each consume a full day and record it → `INTENT-001`, `REST-001`, `TRADE-001`, `HUNT-001`, `RECORD-001`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1303` phase must not mutate state it does not own → `ARCH-004B`, `TEST-001`, `AUDIT-001`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1324` cross-slice influence must flow via derived effects (not ad-hoc mutation) → `WEATHER-002`, `EXEC-001`, `ARCH-004B`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1345` parity-critical rules must be selected via named policy overlay → `GLOBAL-003`, `POLICY-001`, `POLICY-006`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1386` crossing outcomes must include required families (incl. drownings where applicable) → `CROSSING-002`, `AUDIT-009`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1437` policy must define event weights, context multipliers, and nav delay distributions → `POLICY-005`, `ENCOUNTER-003`, `TRAVEL-002`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1444` policy must set `SCORE_POINTS_PER_PERSON_BY_HEALTH` → `SCORE-001`, `POLICY-002`
- `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md:1450` policy must set death-imminent grace days and reset behavior → `HEALTH-005`, `POLICY-005`

### `ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md`

- `ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md:42` `Event.kind` must be mechanical; satire is presentation-only via `ui_surface_hint` → `GLOBAL-002`, `EVENT-001`, `UI-002`
- `ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md:44` narrative rendering must not change outcomes/RNG/phase order → `GLOBAL-002`, `AUDIT-011`
- `ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md:100` `0` sentinel mile markers must be treated as “node absent” → `TRAIL-001`, `TRAIL-002`, `AUDIT-007`
- `ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md:290` crossing outcomes must include required families → `CROSSING-002`, `AUDIT-009`
- `ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md:348` The Dalles gate must block travel beyond node 16 until resolved → `TRAIL-003`, `UI-001`, `AUDIT-008`

### `ENGINE_JOURNEY_CONTROLLER_DIFF.md`

- `ENGINE_JOURNEY_CONTROLLER_DIFF.md:136` satire must be presentation-layer only → `GLOBAL-002`, `UI-002`
- `ENGINE_JOURNEY_CONTROLLER_DIFF.md:137` events must remain mechanically named → `GLOBAL-002`, `EVENT-001`, `EVENT-003`
- `ENGINE_JOURNEY_CONTROLLER_DIFF.md:139` satire must not add hidden modifiers/RNG/reorder phases → `GLOBAL-002`, `AUDIT-011`

## 0) Non-Negotiable Global Requirements (applies to every item)

- [ ] GLOBAL-001 (P0) Preserve determinism and replayability
  - Requirement:
    - The simulation must be deterministic given the same initial state, policy selection, and seed.
    - RNG usage must be phase-scoped and stable across refactors.
  - Spec refs:
    - Systems spec §4.1, §15; Kernel pseudocode “RNG” and “TICK_DAY Pipeline”; Diff “RNG usage is phase-scoped”.
  - Acceptance criteria:
    - For a fixed seed, a full run produces identical `DayRecord` sequences and identical event streams.
    - Tests that assert “deterministic digest” are updated only when intentional behavior changes are approved.

- [ ] GLOBAL-002 (P0) Enforce satire as a presentation-only contract
  - Requirement:
    - All satire/narrative copy must be data-addressed (i18n keys).
    - Satire MUST NOT introduce RNG draws, hidden modifiers, timing changes, or phase reordering.
    - Mechanical event identity must remain stable; satire is applied by UI using `ui_surface_hint`/copy keys.
    - If satire adds extra optional flavor (alternate lines, jokes, “barks”), selection MUST be deterministic from stable inputs
      (e.g., event id + day + locale) and MUST NOT consume RNG.
    - Satire targets systems/incentives (bureaucracy, corruption, media cycles), not protected traits or marginalized groups.
    - The simulation kernel remains value-neutral and mechanical; only the presentation copy varies by locale/theme.
  - Spec refs:
    - Systems spec §2.1 and all “Satire MUST…” bullets; Kernel pseudocode “Satire/presentation contract”; Journey diff “Political Satire Alignment”.
  - Acceptance criteria:
    - Engine layer emits mechanical events with deterministic payloads.
    - UI layer chooses satire text without reading RNG or mutating state.

- [ ] GLOBAL-003 (P0) No piecemeal policy mixing
  - Requirement:
    - Any divergence from OTDeluxe behavior must be a named overlay (e.g., `DystrailLegacyPolicy`) and must not be mixed ad hoc.
  - Spec refs:
    - Systems spec §0 “Any divergence MUST…”, §4.3, §16.
  - Acceptance criteria:
    - It is impossible (by type/config) to accidentally combine OTDeluxe economics with Dystrail illness odds, etc.

---

## 1) Architecture: Single Source of Truth for “One Day” (Kernel owns the day)

- [x] ARCH-001 (P0) Introduce `DailyTickKernel` as the only day orchestrator
  - Requirement:
    - Implement a single kernel entrypoint that executes the normative phase order and returns a structured `DayOutcome`.
    - All callers (web UI, tester/sim harness, any CLI) must invoke this kernel; there must be no alternate “partial day” stacks.
  - Spec refs:
    - Systems spec §14 (phase order), §14.1 (phase ownership), §15 (invariants)
    - Kernel pseudocode “TICK_DAY Pipeline (Normative)”
    - Journey diff “Required OTDeluxe-Parity Order (Normative)”
  - Touchpoints (indicative):
    - `dystrail-game/src/journey/*` (new kernel module)
    - `dystrail-game/src/state.rs` (replace/flatten current orchestration)
    - `dystrail-web/src/app/view/handlers/travel.rs`
    - `dystrail-tester/src/logic/*`
  - Acceptance criteria:
    - Exactly one “advance day” function exists in engine code.
    - Web UI and tester both call the same engine “tick day” path.
    - Phase order matches the spec in every gameplay mode that claims OTDeluxe parity.
  - Determinism notes:
    - Kernel must be the only place that consumes day-level RNG streams; no pre-tick RNG consumption in UI helpers.

- [x] ARCH-002 (P0) Make `start_of_day()` resets-only (remove embedded physics)
  - Requirement:
    - `StartOfDay` must only reset per-day flags/counters, initialize day record, decrement timers/cooldowns as allowed.
    - It must not apply starvation, illness, weather selection, clamps, or other “physics”.
  - Spec refs:
    - Systems spec §14 (StartOfDay vs other ticks), §14.1 ownership table.
    - Journey diff “StartOfDay includes starvation/illness/weather today (ordering issue)”.
  - Current code notes (must change for parity):
    - `dystrail-game/src/state.rs::GameState::start_of_day` currently performs non-reset work, including:
      - `tick_exec_order_state()`
      - `apply_starvation_tick()`
      - `roll_daily_illness()`
      - `apply_deep_aggressive_sanity_guard()`
      - `crate::weather::process_daily_weather(...)`
      - `stats.clamp()`
      - (classic mode) `apply_travel_wear_scaled(1.0)`
  - Acceptance criteria:
    - `start_of_day()` does not:
      - select weather
      - change HP/Sanity/Pants/Supplies beyond defined “reset” semantics
      - roll disease
      - clamp stats as a “physics” step
    - All those actions occur in their dedicated kernel phases.

- [x] ARCH-003 (P0) Eliminate “out-of-kernel daily physics” calls
  - Requirement:
    - Remove or hard-wall any code paths where day physics are applied outside the kernel, including:
      - UI calling helpers that internally call `start_of_day()` + compute miles/encounter chance
      - Session wrappers applying daily effects separately
  - Spec refs:
    - Systems spec §15 invariants (“Weather resolves once per day”, “Supplies burn occurs once per day”, etc.)
  - Acceptance criteria:
    - `JourneySession::tick_day()` and all UI entrypoints do not call “daily physics” functions outside the kernel.
    - Any “preview” computations (for UI) are pure functions that do not mutate state or consume RNG.

- [ ] ARCH-004 (P1) Define explicit “DayInputs / DayEffects / DayEvents / DayRecord”
  - Requirement:
    - Each day must produce:
      - `DayInputs` (player intent + derived conditions)
      - `DayEffects` (resource/health/progress deltas)
      - `DayEvents[]` (structured events)
      - `DayRecord` (audit/replay snapshot)
  - Spec refs:
    - Systems spec §1.1 “Canonical Day Loop”.
  - Acceptance criteria:
    - Kernel returns a `DayOutcome` containing these artifacts (directly or via fields).
    - Replay/audit can reconstruct day outcomes without reading UI logs.

- [ ] ARCH-004B (P0) Enforce phase ownership boundaries structurally (compile-time preferred)
  - Requirement:
    - The systems spec defines strict phase ownership (WeatherTick owns weather/accumulators/effects; HealthTick owns health/afflictions/strain; etc.).
    - Implementation must enforce this beyond convention by structuring code so phases cannot easily mutate foreign slices:
      - Prefer: split `GameState` into explicit sub-structs (`WeatherState`, `HealthState`, `TravelState`, `EncounterState`, `CrossingState`, etc.) and pass only the owned `&mut` slice to each tick.
      - Alternative: a `KernelStateView`/borrow-checked wrapper that exposes only the allowed mutable accessors per phase.
    - Any cross-slice influence must occur via derived-effect structs (e.g., `WeatherEffects`, `ExecOrderEffects`) that downstream phases read.
  - Spec refs:
    - Systems spec §14.1 Phase Ownership; §15 invariants; Weather fan-out requirement §5.1–§5.2.
  - Acceptance criteria:
    - There is no “grab &mut GameState and tweak anything” inside a phase; phases receive only the state they own.
    - Tests in `TEST-001` can be narrow (focused) because the architecture prevents most ownership violations by construction.

- [x] ARCH-005 (P0) Refactor `GameState::travel_next_leg()` into a thin kernel wrapper (or delete it)
  - Requirement:
    - `dystrail-game/src/state.rs::GameState::travel_next_leg` currently orchestrates the day; it MUST be replaced by the kernel phase pipeline.
    - Post-refactor, `travel_next_leg()` must either:
      - (A) become a thin wrapper that delegates to `DailyTickKernel::tick_day(...)`, or
      - (B) be removed entirely in favor of a single `tick_day` API used by all callers.
    - All phase logic currently embedded in `travel_next_leg()` must be moved into the kernel phases, including:
      - boss gating (`guard_boss_gate`)
      - pre-travel checks (`pre_travel_checks`)
      - breakdown and travel block handling
      - encounter flow
      - endgame/crossing/record-day ordering
  - Spec refs:
    - Systems spec §14 (phase list), §14.1 (ownership), §15 (invariants); Journey diff “Current travel_next_leg() Order” + “Required Order”.
  - Touchpoints (concrete):
    - `dystrail-game/src/state.rs::GameState::travel_next_leg`
    - `dystrail-game/src/state.rs::GameState::guard_boss_gate`
    - `dystrail-game/src/state.rs::GameState::pre_travel_checks`
    - `dystrail-game/src/state.rs::GameState::vehicle_roll` / `resolve_breakdown` / `handle_vehicle_state`
    - `dystrail-game/src/state.rs::GameState::handle_travel_block`
    - `dystrail-game/src/state.rs::GameState::process_encounter_flow`
    - `dystrail-game/src/endgame.rs::run_endgame_controller`
    - `dystrail-game/src/state.rs::GameState::handle_crossing_event`
    - `dystrail-game/src/state.rs::GameState::record_travel_day` + `failure_log_key` + `end_of_day`
  - Acceptance criteria:
    - Kernel is the only orchestrator of the day; `travel_next_leg()` no longer contains phase-order decisions.
    - All early returns in the day tick still produce a `DayRecord` and `events[]` consistent with the spec.

- [x] ARCH-006 (P0) Refactor `JourneyController::tick_day()` to call the kernel (stop per-tick state mutation)
  - Requirement:
    - `dystrail-game/src/journey/mod.rs::JourneyController::tick_day` currently injects config into `GameState` and then calls `travel_next_leg()`.
    - Post-refactor, controller tick MUST:
      - delegate to `DailyTickKernel::tick_day(...)`, and
      - stop mutating gameplay state as an implicit side effect of ticking (config injection becomes explicit inputs).
    - Mechanical policy selection MUST be explicit and must not be conflated with:
      - existing `PolicyId` (Classic/Deep family configs)
      - existing `StrategyId` overlays (Balanced/Aggressive/etc.)
  - Spec refs:
    - Systems spec §4.3 (mechanical overlay separation), §14 (kernel ownership), Journey diff “injects config into GameState”.
  - Touchpoints (concrete):
    - `dystrail-game/src/journey/mod.rs::JourneyController::tick_day`
    - `dystrail-game/src/journey/mod.rs::PolicyId` / `StrategyId` / `policy_catalog()`
    - `dystrail-game/src/state.rs` fields currently overwritten in `tick_day` (journey cfg injections)
  - Acceptance criteria:
    - The controller provides explicit inputs (policy selection, strategy overlay, seed/RNG bundle, endgame config) to the kernel.
    - Switching policies is declarative, not a side-effect of “ticking once”.

- [x] BOSS-001 (P0) Implement `BossGateTick` (phase 5) and migrate current boss-gate semantics
  - Requirement:
    - Boss gating MUST occur after WeatherTick + SuppliesBurnTick + HealthTick, and before IntentTick.
    - Boss gating must:
      - block travel for the day (miles = 0)
      - preserve deterministic state so resuming the boss UI does not reroll day physics
      - avoid consuming RNG (unless explicitly specified later)
    - Under `OTDeluxe90sPolicy`, boss gating is disabled unless the run explicitly opts into Dystrail campaign rules.
  - Spec refs:
    - Systems spec §14 phase 5; §13.7 hard-stop rule; Journey diff “Boss gate should occur after daily physics”.
  - Touchpoints (concrete):
    - `dystrail-game/src/state.rs::GameState::guard_boss_gate`
    - `dystrail-game/src/state.rs::boss` state
  - Acceptance criteria:
    - In parity mode, boss gating cannot occur “before weather/consumption/health”.
    - If boss gate blocks, the day is recorded as a NonTravel day with an explicit reason tag/event.

---

## 2) State Model, Calendar, and Core Data Structures

- [x] STATE-001 (P0) Implement the required OTDeluxe day state (minimum MECC parity fields)
  - Requirement:
    - Under `OTDeluxe90sPolicy`, the engine MUST have explicit state (fields or a single nested struct) for the minimum parity model:
      - `day` (day counter)
      - `miles_traveled` (total progress)
      - `region/terrain` and `season` (derived from date/calendar)
      - `party` (members) and `party_alive` (alive count)
      - `health_general` (0 best; higher worse; policy-defined thresholds)
      - `death_imminent_days_remaining` (policy-defined semantics; see HEALTH-005)
      - `oxen_healthy` and an “effective oxen” computation including sick ox weight
      - Itemized inventory:
        - `food_lbs`, `bullets`, `clothes_sets`, `cash_cents`
        - `spares_wheels`, `spares_axles`, `spares_tongues`
      - Player choice inputs:
        - `pace` (Steady/Strenuous/Grueling)
        - `rations` (Filling/Meager/Bare Bones)
      - Weather:
        - `weather_today` (temp, precip, label)
        - `rain_accum`, `snow_depth`
      - Travel gating:
        - `wagon_state` (Moving/Stopped/Resting/Delayed/Blocked)
        - `delay_days_remaining` / `blocked_days_remaining` (separate counters if needed)
        - `ferry_wait_days_remaining` (for ferry queues; distinct from navigation delays so UI/logs are unambiguous)
      - Crossings:
        - `river_state` (width/depth/swiftness + bed type) when at a river node
        - pending crossing choice state so the game can stop and resume deterministically
      - Route graph:
        - `route_variant` (main/sublette/dalles/both)
        - `current_node_index` derived from mile markers (see TRAIL-001)
        - pending route-variant choice state (Sublette at South Pass; Dalles shortcut at Blue Mountains) so the game can stop and resume deterministically
        - pending “The Dalles final route choice” state (raft vs Barlow) so the game can stop and resume deterministically
      - `flags/mods` container for policy modifiers (including occupation perks and (optionally) exec orders).
  - Spec refs:
    - Systems spec §3.1 (required simulation state), §8.5 (route variants), §9 (river state), §11 (inventory units), §12 (scoring).
  - Acceptance criteria:
    - The OTDeluxe90s state can be serialized, replayed, and used for scoring without referencing Dystrail-only abstractions.

- [x] STATE-002 (P0) Implement party member condition state (per-person sickness/injury + death)
  - Requirement:
    - Represent party members and per-member health status at least at the granularity needed for Deluxe:
      - alive/dead
      - sick/injured active status and remaining duration (or a `DiseaseInstance` list per member)
    - Affliction selection chooses a *specific* member; repeat selection while active kills that member.
    - Party member status must feed:
      - speed penalty (`M_party_sick`)
      - hunting carry capacity (`100 * alive_party_members`)
      - endgame score “party_alive” and per-person points.
  - Spec refs:
    - Systems spec §6.3 (choose person + repeat-kills), §6.4 (sick speed penalty), §11.8.2 (carry cap), §12.4 (score).
  - Acceptance criteria:
    - Party-alive count is authoritative and consistent across health, hunting, and scoring.

- [x] STATE-003 (P0) Implement oxen state + “effective oxen” computation (Deluxe mobility)
  - Requirement:
    - Track oxen as a quantity (including “sick” vs “healthy” if modeled explicitly).
    - Implement `effective_oxen = healthy_oxen + sick_oxen * SICK_OX_WEIGHT`.
    - Travel viability and speed scaling use “effective oxen” (see TRAVEL-006/007).
  - Spec refs:
    - Systems spec §8.0 (viability gates), §8.2 (oxen scaling), policy table (`SICK_OX_WEIGHT`, `OXEN_MIN_TO_MOVE`, `OXEN_MIN_FOR_BASE`).
  - Acceptance criteria:
    - A run with 0 effective oxen hard-blocks travel until corrected (Deluxe string semantics).

- [x] STATE-004 (P1) Implement river state persistence (depth/width/swiftness/bed type)
  - Requirement:
    - When arriving at a river node, compute and persist a `river_state` snapshot used by the crossing UI.
    - River state derivation must be deterministic and depend on:
      - river minimums
      - `rain_accum`
      - season/month highs (March/April) and summer decline behavior
  - Spec refs:
    - Systems spec §9.2; crossing pseudocode; “Weather → RiverDepthDelta” fan-out.
  - Acceptance criteria:
    - Repeatedly opening/closing the crossing UI does not reroll river conditions.

- [x] TIME-001 (P0) Implement calendar/date and season derivation for Deluxe parity
  - Requirement:
    - Track a calendar date (or at minimum month + day-in-month) so that:
      - `season` is derived deterministically from date
      - weather and river highs can depend on month/season (even if OT climate stations are optional for v1)
  - Spec refs:
    - Systems spec §3.1 (season derived from date), §9.2 (March/April highs), §5.2 (labeling), §5.1 (stations model optional).
  - Current code notes (must change for parity):
    - `dystrail-game/src/state.rs::Season::from_day` currently uses fixed 45-day seasons and does not track month/day-in-month,
      so it cannot express March/April river highs or month-scoped climate behavior.
  - Acceptance criteria:
    - Advancing N days always advances date deterministically and recomputes season accordingly.

- [x] PACE-001 (P0) Implement OTDeluxe pace model (8/12/16 hours + multipliers + health/fatigue effects)
  - Requirement:
    - `pace` must be a Deluxe-style enum:
      - Steady (8 hours/day), Strenuous (12), Grueling (16)
    - Travel speed uses multipliers: `1.0/1.5/2.0` (policy-defined).
    - Pace also contributes to:
      - health penalty (`H_pace`)
      - fatigue/wear/strain as a Dystrail integration (policy-defined mapping, but deterministic).
  - Spec refs:
    - Systems spec §4.3.1 (`PACE_MULT[...]`), §6.2 (`H_pace`), §8.2 (multipliers), sources note in §0.
  - Acceptance criteria:
    - Pace affects both progress and health/strain in the same day’s causal chain (no delayed/hidden updates).

- [x] RATIONS-001 (P0) Implement OTDeluxe rations model and ensure it affects BOTH consumption and health
  - Requirement:
    - `rations` must be a Deluxe-style enum: Filling / Meager / Bare Bones.
    - Rations must affect:
      - daily food/supplies burn (consumption)
      - health penalty (`H_rations`)
  - Spec refs:
    - Systems spec §7 (rations affect both food consumption and health penalties), §6.2 (`H_rations`).
  - Acceptance criteria:
    - It is impossible for rations to change consumption without also potentially affecting health.

- [x] OCC-001 (P0) Implement Deluxe occupations as first-class run configuration (not Dystrail personas)
  - Requirement:
    - Implement the Deluxe occupation set as a first-class enum/config under OTDeluxe90sPolicy:
      - `banker`, `doctor`, `merchant`, `blacksmith`, `carpenter`, `saddlemaker`, `farmer`, `teacher`
    - Each occupation must have policy-defined, Deluxe-parity properties (from the OTDeluxe90sPolicy table):
      - starting cash in dollars (`OCC_STARTING_CASH_DOLLARS[...]`, converted to `cash_cents = dollars * 100`)
      - final score bonus multiplier (`OCC_FINAL_BONUS_MULT[...]`)
      - qualitative advantages represented by explicit perk hooks:
        - doctor fatality reduction
        - blacksmith/carpenter repair-success improvement
        - farmer mobility-failure reduction
    - Occupations must be selectable at run start in OTDeluxe90s mode, separate from Dystrail persona selection.
  - Spec refs:
    - Systems spec policy table (`OCCUPATIONS`, `OCC_STARTING_CASH_DOLLARS`, `OCC_FINAL_BONUS_MULT`, perk defaults).
  - Acceptance criteria:
    - Under OTDeluxe90sPolicy, starting cash and score multiplier match the spec tables.

---

## 3) RNG: Domain Streams + Phase-Scoped Consumption

- [x] RNG-001 (P0) Expand `RngBundle` to full domain streams required by spec
  - Requirement:
    - Provide at least these streams with stable seeding:
      - `weather`, `health`, `travel`, `events`, `crossing`, `trade`, `hunt`, `vehicle/breakdown`, `encounter`
    - (Dystrail extension) Add a dedicated `boss` stream so boss outcomes never couple to encounter/event selection.
  - Spec refs:
    - Systems spec §4.1; Kernel pseudocode “RNG”.
  - Implementation notes (status):
    - `dystrail-game/src/journey/mod.rs::RngBundle` now exposes the full stream set, including `events`, `trade`, `hunt`, and `boss`.
    - All streams are wrapped in `CountingRng` so draw counts are available for determinism debugging.
  - Acceptance criteria:
    - The engine exposes these streams and kernel phases use only their designated streams.
    - Each stream reports draw counts (or equivalent instrumentation) for determinism debugging.

- [x] RNG-002 (P0) Make WeatherTick consume `rng.weather()` (not travel RNG)
  - Requirement:
    - Weather selection and any weather-related stochastic decisions must come from `rng.weather()` only.
  - Spec refs:
    - Systems spec §14 phase 2 “WeatherTick (rng.weather)”.
  - Acceptance criteria:
    - No weather-related code uses `rng.travel()` or `rng.encounter()`.
    - Weather outcomes are identical regardless of whether travel computations occur.

- [x] RNG-003 (P0) Make afflictions/disease consume `rng.health()` only
  - Requirement:
    - Affliction roll, disease selection, duration rolls, and per-day disease tick stochastic elements must come from `rng.health()`.
  - Spec refs:
    - Systems spec §14 phase 4 “HealthTick (rng.health)” and §13.5.
  - Acceptance criteria:
    - Disease/affliction outcomes are identical even if travel RNG draw counts change.

- [x] RNG-004 (P0) Introduce `rng.events()` and fix draw order for navigation + non-navigation events
  - Requirement:
    - Navigation hard-stops are applied in ComputeMilesToday using `rng.events()` in a fixed, documented order.
    - RandomEventTick uses `rng.events()` (same stream, but separate phase with fixed draw ordering).
    - Any “global” random subsystems that are not encounter selection (e.g., exec orders, ally attrition) MUST also
      draw from `rng.events()` so encounter selection remains independent.
  - Spec refs:
    - Systems spec §14 phase 10 and 14; Kernel pseudocode shows separate navigation and non-navigation events.
  - Acceptance criteria:
    - A regression test can assert “given this seed and this day context, the selected event is X”.
    - Event selection changes only if the events stream is intentionally altered.
  - Progress:
    - [x] RNG-004A Route “global random subsystems” (non-encounter) to `rng.events()`
      - Requirements:
        - Exec order selection/duration RNG uses `rng.events()`.
        - Ally attrition RNG uses `rng.events()`.
      - Acceptance criteria:
        - `dystrail-game/src/state.rs::GameState::tick_exec_order_state` uses `events_rng()`.
        - `dystrail-game/src/state.rs::GameState::tick_ally_attrition` uses `events_rng()`.
    - [x] RNG-004B Route navigation hard-stops + RandomEventTick to `rng.events()` with fixed draw ordering

- [x] RNG-005 (P1) Add optional runtime “Phase RNG Guard”
  - Requirement:
    - Detect cross-phase RNG usage (e.g., WeatherTick accidentally calling `rng.travel()`).
  - Acceptance criteria:
    - In debug builds/tests, the engine can assert the active phase and disallow unexpected stream usage.

- [x] RNG-006 (P1) Document and enforce the phase→RNG-stream contract (including multi-stream phases)
  - Requirement:
    - Maintain a single source of truth that maps:
      - each kernel phase → allowed RNG stream(s)
      - fixed draw ordering when a phase uses >1 stream (e.g., ComputeMilesToday uses `rng.travel` then `rng.events`, in a documented order)
    - This contract must be enforced in tests and ideally in debug builds.
  - Spec refs:
    - Systems spec §4.1 refinement (“phase may only consume listed stream(s)… draw order must be fixed”) and §14 phase list.
  - Acceptance criteria:
    - A change to RNG usage requires updating the contract and associated tests (no “silent drift”).

---

## 4) Mechanical Policy Layer: `OTDeluxe90sPolicy` (separate from Dystrail strategies)

- [ ] POLICY-001 (P0) Implement a first-class mechanical policy selection system
  - Requirement:
    - Add a dedicated mechanical policy layer with explicit selection (e.g., `OTDeluxe90sPolicy` vs `DystrailLegacyPolicy`).
    - This must be distinct from existing difficulty/strategy overlays (Balanced/Aggressive/Conservative/ResourceManager).
  - Spec refs:
    - Systems spec §4.3, §16; Journey diff “No piecemeal mixing”.
  - Acceptance criteria:
    - A single field/config selects the mechanical overlay; every parity-critical rule is sourced from it.
  - Progress:
    - [x] POLICY-001A Plumb `MechanicalPolicyId` through engine state/controller
      - Requirements:
        - Add `MechanicalPolicyId` enum with at least: `DystrailLegacy`, `OtDeluxe90s`.
        - Persist the selected mechanical policy on `GameState` so saves/replays cannot silently drift.
        - Ensure `JourneyController::tick_day` writes `state.mechanical_policy` every tick.
      - Acceptance criteria:
        - `JourneySession::from_state` uses the saved mechanical policy when rebuilding the controller.
        - Existing Dystrail behavior remains unchanged under `DystrailLegacy`.
    - [ ] POLICY-001B Route parity-critical mechanics through the selected overlay
      - Requirements:
        - OTDeluxe and Dystrail mechanics must be cleanly separated; no cross-reading legacy constants.
      - Acceptance criteria:
        - Under `OtDeluxe90s`, all parity-critical constants/curves come from `OTDeluxe90sPolicy` (not legacy config).

- [ ] POLICY-002 (P0) Represent every “policy-defined” value as an explicit parameter (no hidden constants)
  - Requirement:
    - All items marked “policy-defined”, “must be set”, or “still open” in the systems spec must exist as explicit fields in policy config.
  - Spec refs:
    - Systems spec §4.3.1 policy table and all “MUST be expressed as policy parameters” language.
  - Acceptance criteria:
    - No parity-critical numeric remains “implied by code”; policy config is the single place to change them later.
  - Progress:
    - [x] POLICY-002A Add an `OtDeluxe90sPolicy` data module with explicit parameters and defaults
      - Requirements:
        - Introduce a dedicated data type for the OTDeluxe90s mechanical overlay (do not reuse Dystrail strategy overlays).
        - Include explicit policy fields for parity-critical knobs, even when the value is “policy-defined”.
      - Acceptance criteria:
        - `dystrail-game/src/mechanics/otdeluxe90s.rs` provides `OtDeluxe90sPolicy::default()` without reading any runtime state.
        - Types for OTDeluxe enums (pace/rations/occupations/trail variants) exist and are serializable.
    - [ ] POLICY-002B Route parity-critical computations through `OtDeluxe90sPolicy` (no legacy constants)

- [ ] POLICY-002C (P1) Support per-region and per-season overrides inside the mechanical policy
  - Requirement:
    - The policy layer must support overrides keyed by:
      - region/terrain band
      - season/month band
    - Overrides must be able to target at least:
      - travel multipliers (terrain/season penalties)
      - event/encounter weights and caps
      - weather or weather-effect adjustments (if applicable)
      - disease/affliction weights (if applicable)
    - Overrides are *mechanical* and therefore must be part of the policy overlay, not UI flavor.
  - Spec refs:
    - Kernel pseudocode “PolicySet { … per_region_overrides, per_season_overrides }”; Systems spec policy/overlay principles (§4.3, §15).
  - Acceptance criteria:
    - A single policy can define “same rule, different region/season values” without branching logic in the kernel.

- [ ] POLICY-003 (P0) Embed the extracted Deluxe constants into OTDeluxe90sPolicy
  - Requirement:
    - Policy must include (at minimum) the EXE-extracted and locked decisions:
      - Store base prices (cents), caps, price multipliers, store node indices
      - Trail mile markers (route variants)
      - Crossing costs (ferry $5, guide 3 clothes)
  - Spec refs:
    - Systems spec §4.3.1 (policy table), §8.5 (mile markers), §10 (store), §9 (crossings).
  - Acceptance criteria:
    - Policy values match the spec exactly; any code using different values is removed or gated to other policies.
  - Progress:
    - [x] POLICY-003A Embed EXE-extracted store/trail/crossing constants in `OtDeluxe90sPolicy::default()`
      - Requirements:
        - Store price multiplier schedule is treated as normative (first 18 entries align to node indices 0..17).
        - Oregon City is presentation-only (no additional node beyond Willamette Valley).
      - Acceptance criteria:
        - `OtDeluxe90sPolicy::default()` includes:
          - `STORE_BASE_PRICE_CENTS[...]`
          - `STORE_PRICE_MULT_PCT_BY_NODE[...]`
          - `STORE_MAX_BUY[...]`
          - Trail mile marker lists (main + 3 route variants with 0 sentinels)
          - `FERRY_COST_CENTS` and `GUIDE_COST_CLOTHES`
        - A future OTDeluxe kernel can consume these constants without any additional extraction step.

- [ ] POLICY-004 (P0) Policy must define the affliction odds curve and odds driver
  - Requirement:
    - `OTDeluxe90sPolicy` uses `health_general` via `AFFLICTION_CURVE_PWL` (clamped to `P_AFFLICTION_MAX`).
    - Dystrail overlays may use `general_strain` with a separate `f_strain_to_prob` mapping.
  - Spec refs:
    - Systems spec §13.5 (odds driver), §4.3.1 (AFFLICTION_CURVE_PWL).
  - Acceptance criteria:
    - Odds driver is chosen explicitly by policy; cannot drift due to incidental code paths.

- [ ] POLICY-005 (P1) Policy must own “Deluxe-unknown but parity-critical” knobs (instrumented for later fitting)
  - Requirement:
    - Define policy placeholders + logging for later empirical fit/extraction:
      - ferry wait distribution (default uniform 0..6)
      - ferry accident outcomes/weights (non-lethal default)
      - guide loss multiplier default
      - navigation delay distributions
      - event base weights + context multipliers
  - Spec refs:
    - Systems spec §16 checklist items (e.g., ferry mechanics, event weights, delays).
  - Acceptance criteria:
    - Debug telemetry records sampled values so they can be fit later without changing kernel logic.

- [ ] POLICY-006 (P0) Audit and fence off Dystrail-only “guardrails” and meta-modifiers from OTDeluxe90sPolicy
  - Requirement:
    - The current engine contains multiple Dystrail-only mechanics that would violate Deluxe parity if they remain active under OTDeluxe90sPolicy.
    - Implementation MUST inventory all such mechanics and ensure they are:
      - disabled under OTDeluxe90sPolicy, or
      - re-expressed as explicit, documented policy parameters (if intentionally kept).
    - At minimum, audit and gate the following existing behaviors:
      - “behind schedule” modifiers (e.g., `behind_schedule_multiplier()` affecting disease/encounters/travel)
      - Deep/Classic “failsafe” travel credits and repair guards (e.g., `apply_delay_travel_credit`, limp/jury-rig guards)
      - start-of-day special-case sanity/health guards (e.g., deep aggressive sanity guard)
      - aggressive/conservative mode-specific multipliers embedded in breakdown/travel odds
      - any “partial ratio floors” or “distance caps” that leak miles on NonTravel days
    - The audit result (list of gated features and their policy) MUST be written down in a developer-facing document or module-level doc comment so parity decisions remain traceable.
  - Spec refs:
    - Systems spec §0 (“Any divergence MUST be explicit overlay”), §13.7 hard-stop rule, §15 invariants; Journey diff “Hard-stop semantics” and “No piecemeal mixing”.
  - Touchpoints (concrete, non-exhaustive):
    - `dystrail-game/src/state.rs`:
      - `behind_schedule_multiplier`
      - `apply_delay_travel_credit`
      - deep/Classic field-repair guards and emergency limp guards
      - `apply_deep_aggressive_sanity_guard`
      - any distance cap / ratio floor fields (`distance_cap_today`, partial ratio)
    - `dystrail-game/src/state.rs::GameState::vehicle_roll` (aggressive/conservative overrides)
    - `dystrail-game/src/state.rs::GameState::roll_daily_illness` (Dystrail-only disease chance modifiers)
  - Acceptance criteria:
    - Under OTDeluxe90sPolicy, a day’s outcomes depend only on OTDeluxe policy parameters + kernel phases, not hidden Dystrail-only assists.
    - Under DystrailLegacyPolicy, existing behaviors remain available and deterministic.

- [x] EXEC-001 (P1) Integrate exec orders as deterministic, phase-scoped modifiers (Dystrail extension)
  - Requirement:
    - Exec orders must be modeled as:
      - a deterministic state machine (active order + remaining duration + cooldown)
      - a derived `ExecOrderEffects` struct that downstream phases read (mirrors WeatherEffects fan-out pattern)
    - Exec order selection randomness must be assigned to a named RNG stream and phase (recommended: RandomEventTick via `rng.events()`).
    - Exec orders must be able to modify (as the spec requires):
      - travel multipliers
      - breakdown odds
      - encounter odds
      - supplies burn
      - strain
  - Spec refs:
    - Systems spec §13.2 (exec orders and policies), §14 (phase ownership boundaries).
  - Current code notes (completed):
    - `tick_exec_order_state` now populates `ExecOrderEffects` and downstream phases consume effects without direct stat mutation.
  - Acceptance criteria:
    - Exec orders do not mutate downstream state directly outside their owning phase; they produce derived effects.
    - Removing exec orders does not change RNG consumption in unrelated phases.

---

## 5) Event Bus + Day Records (Event-Sourced Simulation)

- [ ] EVENT-001 (P0) Implement structured `Event` and emit events for all state changes
  - Requirement:
    - Replace “log key as truth” with structured events.
    - Logs/UI strings are derived from events, not driving logic.
  - Spec refs:
    - Systems spec §4.2; Kernel pseudocode Event struct.
  - Acceptance criteria:
    - A day outcome contains `events: Vec<Event>`.
    - UI can render the day using event-to-copy mapping.
  - Progress:
    - [x] EVENT-001A Add core `Event` types and plumb `events[]` through `DayOutcome`
      - Acceptance criteria:
        - `dystrail-game/src/journey/event.rs` defines `Event`, `EventId`, `EventKind`, and `UiSurfaceHint`.
        - `dystrail-game/src/journey/mod.rs::DayOutcome` includes `events: Vec<Event>`.
        - `dystrail-web/src/app/view/handlers/travel.rs` renders day output from `outcome.events` (with a fallback to `log_key` during transition).
    - [ ] EVENT-001B Emit structured events for all state changes (retire `log_key` as the only truth)

- [ ] EVENT-002 (P0) Implement “explainable event selection telemetry” (`EventDecisionTrace`)
  - Requirement:
    - Whenever a random event/encounter is selected, capture “why” (base weight, multipliers, context inputs).
  - Spec refs:
    - Systems spec §9.4 “Implementation-binding requirement (Q15): event selection MUST produce explainable telemetry”.
  - Acceptance criteria:
    - Telemetry is available in debug builds/tests and can be optionally recorded in `DayRecord`.
  - Progress:
    - [x] EVENT-002A Add `EventDecisionTrace` data type (no emit sites yet)
    - [ ] EVENT-002B Populate traces from encounter/event selection and store them in `DayOutcome` (and optionally `DayRecord`)
      - Progress:
        - Encounter selection now emits traces and `JourneyController::tick_day()` surfaces them via `DayOutcome.decision_traces`.
        - Remaining: RandomEventTick + any other weighted pools must emit traces too (with fixed draw ordering).

- [ ] EVENT-003 (P1) Provide stable event IDs and tags
  - Requirement:
    - Events must include stable identifiers for replay and log correlation.
  - Acceptance criteria:
    - A replay can compare event streams for equality (including IDs) when desired.
  - Progress:
    - [x] EVENT-003A Add `EventId { day, seq }` and `tags: DayTagSet` on `Event` (deterministic, non-RNG)
    - [ ] EVENT-003B Ensure all event emit sites assign stable `seq` ordering per day and preserve ordering across refactors

---

## 6) Weather System (Deluxe causality, Dystrail generator)

- [ ] WEATHER-001 (P0) Make WeatherTick the root cause of the day (phase order)
  - Requirement:
    - Weather selection happens before supplies burn, health, breakdown roll, encounters, and crossings.
  - Spec refs:
    - Systems spec §5 and §14 phase 2; Journey diff delta “Weather must be the root cause”.
  - Current code notes (must change for parity):
    - `dystrail-game/src/state.rs::GameState::start_of_day` calls `crate::weather::process_daily_weather(...)` today (too early/embedded).
    - `dystrail-game/src/weather.rs::select_weather_for_today` now consumes `rng.weather()` (fixed), but is still invoked from `start_of_day()` instead of a dedicated WeatherTick phase.
  - Acceptance criteria:
    - Kernel phase order enforces this; no other phase selects or mutates `weather_state.today`.

- [ ] WEATHER-002 (P0) Introduce `WeatherEffects` fan-out struct and ensure downstream phases read it
  - Requirement:
    - Weather must produce a single “fan-out” object that covers the full causal footprint:
      - travel speed/progress deltas
      - supplies/consumption deltas
      - health/strain deltas
      - encounter/event probability deltas
      - breakdown probability deltas
      - river/crossing deltas (either explicit `river_depth_delta` or deterministic updates to `rain_accum`/`snow_depth` that river state derivation reads)
  - Spec refs:
    - Systems spec §5.1–§5.2, §14.1 ownership boundaries.
  - Acceptance criteria:
    - Encounter chance and breakdown chance are influenced through `WeatherEffects`, not ad hoc additions.

- [ ] WEATHER-003 (P0) Add precipitation accumulators (`rain_accum`, `snow_depth`) and hooks for snow slowdown
  - Requirement:
    - State must carry `snow_depth` and the slowdown hook `M_snow(snow_depth)` even if snow is dormant in v1.
  - Spec refs:
    - Systems spec requirement at line ~380 (“MUST carry snow_depth and slowdown hook”).
  - Acceptance criteria:
    - WeatherTick updates accumulators; TravelTick can consult `snow_depth` via policy when enabled.

- [ ] WEATHER-004 (P1) Add WeatherModel interface with `DystrailRegionalWeather` default; keep `OTDeluxeStationsWeather` optional
  - Requirement:
    - Implement `WeatherModel` with at least:
      - `generate_weather_today(ctx, rng.weather())`
      - `compute_weather_effects(ctx, weather_today)`
    - Provide:
      - `DystrailRegionalWeather` using existing `weather.json`
      - `OTDeluxeStationsWeather` interface stub (tables/procedure optional future)
  - Spec refs:
    - Systems spec §5.1 “Normative abstraction: WeatherModel … Required implementations”.
  - Acceptance criteria:
    - Kernel does not depend on hardcoded weather logic; it calls the WeatherModel.
    - Optional station model can be wired later without touching kernel.

- [ ] WEATHER-005 (P1) Implement MECC/Deluxe weather report labels (precip-first; otherwise temperature bands)
  - Requirement:
    - Given a sampled daily weather (temp + precip), derive a label deterministically:
      - If precipitation present: rainy / snowy / very rainy / very snowy (thresholds policy-defined if not emitted directly)
      - Else temperature band:
        - very hot (>90F), hot (70–90), warm (50–70), cool (30–50), cold (10–30), very cold (<10)
  - Spec refs:
    - Systems spec §5.2.
  - Acceptance criteria:
    - Labels are derived without additional RNG draws and are stable for replay/audit.

- [ ] WEATHER-006 (P1) Implement rain/snow accumulation + evaporation/melt hooks (policy-defined rates)
  - Requirement:
    - Daily precipitation updates:
      - `rain_accum` and `snow_depth`
      - daily evaporation for both
      - snow melt on warm days converting to rain/water accumulation
    - Rates and thresholds are policy-defined (Deluxe-exact if later extracted).
  - Spec refs:
    - Systems spec §5.3.
  - Acceptance criteria:
    - Accumulators evolve deterministically day-to-day and feed into river state and snow slowdown.

---

## 7) Supplies Burn (“Eat”) + Starvation

- [ ] SUPPLIES-001 (P0) Implement explicit SuppliesBurnTick (phase 3)
  - Requirement:
    - Apply daily consumption exactly once per day, after WeatherTick and before HealthTick.
    - Must run even on Rest/Trade/Hunt days.
  - Spec refs:
    - Systems spec §14 phase 3 and invariants in §15.
  - Acceptance criteria:
    - There is no way to skip supplies burn by resting/hunting/trading.

- [ ] SUPPLIES-002 (P0) Move starvation mechanics out of StartOfDay and make them a post-burn consequence
  - Requirement:
    - Starvation effects/timers must be advanced as part of the supplies burn/health flow, not in `start_of_day()`.
  - Spec refs:
    - Systems spec §14 ownership and invariants.
  - Current code notes (must change for parity):
    - `dystrail-game/src/state.rs::GameState::apply_starvation_tick` is currently called from `start_of_day()`.
  - Acceptance criteria:
    - Starvation tick occurs in SuppliesBurnTick and/or HealthTick only.

- [ ] SUPPLIES-003 (P1) Align Dystrail “supplies” vs OT “food_lbs” semantics under policy
  - Requirement:
    - OTDeluxe90sPolicy requires itemized food (lbs) for store, hunting carry cap, and scoring.
    - DystrailLegacy may keep umbrella “supplies”.
  - Spec refs:
    - Systems spec §3.1 (food_lbs), §10 (store), §11 (hunting), §12 (scoring).
  - Acceptance criteria:
    - Under OTDeluxe90sPolicy, `food_lbs` exists and is used for: consumption, hunting, store, scoring.
    - Under non-OT policies, existing “supplies” may remain primary.

- [ ] SUPPLIES-004 (P0) Implement the normative supplies-burn formula (pace/diet/weather/vehicle/exec)
  - Requirement:
    - Supplies burn must be computed as:
      - `base_supplies_burn(region)`
        * `pace_supplies_factor(pace)`
        * `weather_supplies_factor(weather/effects)`
        * `vehicle_supplies_factor(vehicle_state)`
        * `exec_supplies_factor(exec_orders)`
        * `diet_supplies_factor(diet)` (Dystrail mapping) OR `rations_factor(rations)` (OTDeluxe mapping)
    - Under OTDeluxe90sPolicy, apply equivalent itemized food burn on `food_lbs`, and ensure rations affect both consumption and health.
  - Spec refs:
    - Systems spec §7 (formula and rations semantics).
  - Acceptance criteria:
    - A day trace can explain the exact burn amount from each multiplier term (and it is recorded in DayEffects/telemetry).

---

## 8) Health, Afflictions, and Disease Catalog

- [x] HEALTH-001 (P0) Add `health_general` and implement OTDeluxe90s health arithmetic
  - Requirement:
    - Maintain `health_general` scalar and implement baseline recovery and penalties in policy:
      - `HEALTH_RECOVERY_BASELINE = health_general -= 10` (policy-defined)
      - Label ranges and death threshold at 140 (policy-defined)
  - Spec refs:
    - Systems spec §4.3.1 policy table; §6.1 health thresholds; §16.1 checklist.
  - Acceptance criteria:
    - `health_general` updates occur in HealthTick only and match the policy function.

- [x] HEALTH-002 (P0) Implement the affliction roll semantics (0..40% odds, repeat kills)
  - Requirement:
    - In HealthTick, compute `p_affliction_today` from policy’s odds driver:
      - OTDeluxe90sPolicy: `AFFLICTION_CURVE_PWL(health_general)` clamped to 0..0.40
    - If an already sick/injured target is selected again, they die (repeat-kills).
    - Durations: illness 10 days; injury 30 days (policy-defined).
  - Spec refs:
    - Systems spec §13.5 and §4.3.1 (ILLNESS_DURATION_DAYS, INJURY_DURATION_DAYS, AFFLICTION_REPEAT_KILLS).
  - Current code notes (must change for parity):
    - `dystrail-game/src/state.rs::GameState::roll_daily_illness` currently:
      - runs in `start_of_day()`, before weather/supplies ordering
      - consumes `rng.health()` (fixed), but is still not scoped to an explicit HealthTick phase
      - uses Dystrail-specific chance modifiers (supplies/hp/starvation/behind-schedule), not the OTDeluxe `health_general -> 0..0.40` curve.
  - Acceptance criteria:
    - Afflictions are deterministic with `rng.health()` only.
    - Repeat selection causes death according to policy.

- [x] HEALTH-003 (P0) Implement named `DiseaseCatalog` as data (not hard-coded)
  - Requirement:
    - Provide a catalog of named illnesses/injuries matching Deluxe presentation needs.
    - Each entry must define:
      - stable mechanical `id` (not localized)
      - a `display_key`/i18n key for UI/log rendering (satire-safe wrapper applied at presentation)
      - selection weights
      - duration model (fixed/range/policy)
      - per-day tick deltas
      - fatality model (for doctor perk)
  - Spec refs:
    - Systems spec §6.3.1 “Disease Catalog (OTDeluxe90sPolicy; data-driven)” and schema note.
  - Acceptance criteria:
    - Catalog is data-driven (JSON/TOML/ron) and loaded by engine.
    - Adding a disease does not require code changes.

- [x] HEALTH-004 (P1) Implement `general_strain` as the Dystrail parity scalar (derived, hidden)
  - Requirement:
    - Compute `general_strain` once per day, deterministically, from HP/Sanity/Pants/vehicle/weather/exec/starvation.
    - Must be policy-weighted and not player-visible.
  - Spec refs:
    - Systems spec §13.4, §13.4.1.
  - Acceptance criteria:
    - `general_strain` is recomputed daily in HealthTick and used only as allowed (odds/weights hooks).

- [x] HEALTH-005 (P1) Implement death-imminent timer semantics (Deluxe phrase “within a few days”)
  - Requirement:
    - If `health_general >= HEALTH_DEATH_THRESHOLD`, set/advance a death timer:
      - `DEATH_IMMINENT_GRACE_DAYS` and reset behavior are policy-defined.
  - Spec refs:
    - Systems spec policy table and §16.14/§16.17; explicit MUST at ~1450.
  - Acceptance criteria:
    - The timer behavior is fully deterministic and documented; tests cover reset semantics.

- [x] HEALTH-006 (P1) Implement occupation perk hooks for fatality/repair/mobility
  - Requirement:
    - Doctor fatality multiplier affects relevant fatality checks (disease complications, repeat-kills if applicable).
    - Blacksmith/Carpenter repair success multiplier affects repair outcomes.
    - Farmer mobility failure multiplier affects ox/mobility analogue outcomes.
  - Spec refs:
    - Systems spec policy table “OCC_* defaults”.
  - Acceptance criteria:
    - Perk effects are applied only via policy hooks; no hard-coded profession if/else in the kernel.

- [x] HEALTH-007 (P0) Implement the full Deluxe-lineage daily health formula (all additive factors)
  - Requirement:
    - HealthTick must apply the normative equation (no double-counting):
      - baseline recovery `health_general -= 10`
      - additive penalties/bonuses from:
        - `H_weather(weather_today)`
        - `H_pace(pace)` (including “resting” semantics if applicable)
        - `H_rations(rations)`
        - `H_clothing(season, clothes_sets)`
        - `H_afflictions(party)` (including active disease ticks)
        - `H_drought(rain_accum, season)`
        - optional event-specific health payloads (`H_event(e)`), but only if the event is not already represented in another term
    - Each `H_*` must be policy-defined or policy-selectable (Deluxe-exact where known; placeholders where unknown).
  - Spec refs:
    - Systems spec §6.2 (formula) and note about event-sourced application order.
  - Acceptance criteria:
    - A single-day trace can account for every change to `health_general` without hidden/duplicate steps.

- [x] HEALTH-008 (P1) Implement `M_party_sick` travel slowdown and keep it consistent with party state
  - Requirement:
    - Each sick party member reduces travel speed by 10%:
      - `M_party_sick(sick_count) = max(0, 1 - 0.10*sick_count)`
    - “Sick count” comes from the per-member condition model (STATE-002).
  - Spec refs:
    - Systems spec §6.4 (explicit multiplier), §8.2 (sick party slowdown).
  - Acceptance criteria:
    - Travel speed reduction matches the multiplier and is deterministic.

- [x] HEALTH-009 (P1) Implement Dystrail health-label derivation via `general_strain_norm` (for UI/scoring parity)
  - Requirement:
    - Compute:
      - `general_strain_norm = clamp(general_strain / STRAIN_NORM_DENOM, 0..1)`
      - map to labels using `STRAIN_LABEL_BOUNDS` (defaults 0.25/0.50/0.75)
    - Use this mapping only where the spec says Dystrail translation is authoritative (UI summaries, score display), and never to “back-drive” HP/Sanity/Pants.
  - Spec refs:
    - Systems spec §13.4.1 and policy table (`STRAIN_NORM_DENOM`, `STRAIN_LABEL_BOUNDS`).
  - Acceptance criteria:
    - For a fixed state, label derivation is deterministic and does not consume RNG.

- [x] HEALTH-010 (P0) Move Dystrail “pre-travel checks” into HealthTick (ally attrition + clamps), and remove duplicate early-exit paths
  - Requirement:
    - The systems spec defines “pre-travel checks” as part of HealthTick (phase 4), not a separate early phase.
    - Migrate:
      - `tick_ally_attrition()` randomness into HealthTick using `rng.health()` (or make ally attrition deterministic under parity policy)
      - `stats.clamp()` into HealthTick as the final step after applying health/affliction/starvation deltas for the day
    - Ensure there is no second/duplicate clamp or early terminal check that causes “double physics” in one day.
  - Spec refs:
    - Systems spec §14 phase 4 (explicit: “tick ally attrition and clamp stats”), §14.1 ownership table; Journey diff notes `pre_travel_checks()` today.
  - Touchpoints (concrete):
    - `dystrail-game/src/state.rs::GameState::pre_travel_checks`
    - `dystrail-game/src/state.rs::GameState::tick_ally_attrition`
    - `dystrail-game/src/state.rs::Stats::clamp` (and all call sites)
  - Acceptance criteria:
    - Under OTDeluxe90sPolicy, health/affliction/consumption deltas occur once, then clamp once, then terminal checks occur in RecordDay + TerminalChecks.
    - Under DystrailLegacyPolicy, existing ally-attrition behavior is preserved but moved into phase 4 to match the kernel contract.

---

## 9) Intent System: Rest, Trade, Hunt, Continue (Day-Atomic)

- [x] INTENT-001 (P0) Introduce `DayIntent` and intent resolution phase (phase 6)
  - Requirement:
    - Implement explicit daily intent resolution:
      - Continue (travel)
      - Rest (N days, day-atomic)
      - Trade (day-atomic)
      - Hunt (day-atomic)
      - CrossingChoicePending (blocks travel until resolved)
  - Spec refs:
    - Systems spec §14 phase 6; Kernel pseudocode intent branches.
  - Acceptance criteria:
    - Intent resolution happens exactly once per day in the kernel and is recorded in DayInputs/DayEvents.

- [x] WAIT-001 (P0) Implement scheduled non-travel “wait day” gates (ferry queues, drying days, etc.)
  - Requirement:
    - Certain mechanics produce forced non-travel days that are not player-chosen intents (Deluxe lineage examples: ferry wait days; drying day after wet goods).
    - Implement a kernel gate (before IntentTick) that:
      - detects an outstanding forced-wait counter (e.g., `ferry_wait_days_remaining`, `drying_days_remaining`)
      - decrements it by 1
      - records a NonTravel day (miles = 0) with an explicit reason tag/event
      - returns without allowing other intents, travel, encounters, or crossings for that day
    - Forced wait days MUST still run the daily root-cause ticks (WeatherTick → SuppliesBurnTick → HealthTick) before terminating the day.
  - Spec refs:
    - Kernel pseudocode “Scheduled non-travel days (ferry queues…)”; Systems spec §9.4.1 (ferry wait) and §9.4.3 (drying day), §14 (root-cause ticks run on non-travel days).
  - Acceptance criteria:
    - While waiting for a ferry, the player cannot “continue traveling” or start hunting/trading; only the wait day is applied.
    - The day record clearly distinguishes forced wait days from voluntary Rest intent days.

- [x] REST-001 (P0) Implement rest as N day-atomic intents (1..9 for Deluxe UX)
  - Requirement:
    - Rest must:
      - advance the day loop (not “free”)
      - run root-cause ticks each day (weather/supplies/health)
      - produce NonTravel day records (miles = 0) under OTDeluxe90sPolicy
  - Spec refs:
    - Systems spec policy table `REST_DAYS_RANGE = 1..9`; §14 phase 6; §7.1 Rest.
  - Acceptance criteria:
    - Rest cannot be implemented as a multi-day “fast forward” that skips per-day weather/health.

- [x] TRADE-001 (P0) Implement trade offers and enforce `TRADE_COST_DAYS = 1`
  - Requirement:
    - Each accepted trade costs a day away from the trail (NonTravel day with root-cause ticks).
    - Trade offer generation uses `rng.trade()` only.
    - Trade must support (at minimum) the Deluxe request categories:
      - oxen, clothing, bullets/ammo, wagon parts (wheel/axle/tongue), food, cash
  - Spec refs:
    - Systems spec policy table `TRADE_COST_DAYS = 1`; Kernel pseudocode trade branch.
  - Acceptance criteria:
    - Accepting a trade advances day exactly once and records a NonTravel day.

- [x] HUNT-001 (P0) Implement hunting as a day-atomic intent and enforce carry cap
  - Requirement:
    - Hunting consumes a full day away from the trail (NonTravel; root-cause ticks still run).
    - Carry cap is `100 * alive_party_members` under OTDeluxe90sPolicy; injuries do not reduce cap.
    - Hunting uses `rng.hunt()` only for stochastic elements.
    - Hunting must enforce Deluxe gating rules:
      - blocked if no bullets
      - blocked by severe weather
      - blocked at some locations (“too many people around”)
      - optional: overhunting reduces future availability (“game becomes scarce”), tracked deterministically
  - Spec refs:
    - Systems spec §11 (hunting day cost + gating + overhunting), policy table `HUNT_COST_DAYS`, `HUNT_CARRY_CAP_LBS`.
  - Acceptance criteria:
    - Food gained is `min(food_shot, carry_cap_lbs)` and additionally limited by wagon food cap if per-item caps apply.

---

## 10) Travel / Progress + Hard-Stops

- [ ] TRAVEL-001 (P0) Implement the normative travel phase order (VehicleTick → TravelBlockTick → EncounterTick → ComputeMilesToday → TravelWearTick)
  - Requirement:
    - Travel cannot occur until after Weather/Supplies/Health + boss gate + intent resolution.
  - Spec refs:
    - Systems spec §14 phases 7–11; Journey diff “Required order”.
  - Acceptance criteria:
    - Refactored pipeline matches the phase list and ownership boundaries.

- [x] TRAVEL-002 (P0) Implement navigation hard-stops and multi-day delays (lost/wrong/impassable/snowbound)
  - Requirement:
    - If any navigation hard-stop occurs:
      - today’s miles are 0
      - wagon/vehicle state is Delayed/Blocked
      - delay days are applied deterministically from a policy-defined distribution
  - Spec refs:
    - Systems spec §8.3/§8.4 and §14 phase 10; Kernel pseudocode nav hard-stops.
  - Acceptance criteria:
    - There is no “delay travel credit” leak for OTDeluxe hard-stops.

- [x] TRAVEL-003 (P0) Enforce “no miles leak” when hard-stopped
  - Requirement:
    - If using `computed_miles_today = max(distance_today, distance_today_raw)`, OTDeluxe90sPolicy MUST ensure:
      - `distance_today_raw == distance_today` whenever travel is allowed
      - both are 0 when travel is hard-stopped
  - Spec refs:
    - Systems spec §13.7 (hard-stop rule + max() warning); explicit MUST at ~1227.
  - Acceptance criteria:
    - Hard-stops cannot yield miles via a “raw distance” fallback.

- [x] TRAVEL-004 (P1) Replace Dystrail-specific stop-caps and travel-credit hacks under OTDeluxe90sPolicy
  - Requirement:
    - Disable or policy-gate mechanisms that turn NonTravel into Partial miles:
      - ratio floors
      - stop caps
      - delay travel credit
      - “limp” travel credit
    - Under OTDeluxe90sPolicy, NonTravel days must remain `miles = 0` unless explicitly specified by Deluxe.
  - Spec refs:
    - Systems spec §14 hard-stop rule; Journey diff “Hard-stops set miles=0”.
  - Acceptance criteria:
    - Under OTDeluxe90sPolicy, blocked days do not advance mileage.

- [ ] TRAVEL-005 (P1) Vehicle system integration as the OT “oxen mobility analogue”
  - Requirement:
    - OTDeluxe “oxen” scaling must be mapped to a Dystrail mobility readiness function under policy:
      - readiness ∈ [0..1]
      - travel speed *= readiness
    - Breakdown roll occurs after WeatherTick and before EncounterTick.
  - Spec refs:
    - Systems spec §13.1 and policy table oxen scaling; §14 phase 7.
  - Acceptance criteria:
    - Vehicle breakdown chance is influenced by weather effects in a single causal chain.

- [x] TRAVEL-006 (P0) Implement Deluxe travel viability gates (no oxen / only ox sick / broken parts)
  - Requirement:
    - Travel must be blocked if:
      - `effective_oxen < OXEN_MIN_TO_MOVE` (default 1.0 effective oxen)
      - “only ox is sick” edge case if modeled (effective oxen collapses to 0 when sick)
      - required wagon part is broken and cannot be repaired/replaced (wheel/axle/tongue)
    - Blocked travel produces:
      - `computed_miles_today = 0`
      - `wagon_state = Blocked`
      - a clear event/log key explaining the requirement to fix it
  - Spec refs:
    - Systems spec §8.0 (Deluxe strings + normative requirement), policy table `OXEN_MIN_TO_MOVE`.
  - Acceptance criteria:
    - These gates are enforced before any miles are credited.

- [ ] TRAVEL-007 (P0) Implement the Deluxe travel formula and multipliers (BASE_MPD=20 model)
  - Requirement:
    - Compute daily miles using the normative Deluxe-lineage formula:
      - base = `BASE_MPD_PLAINS_STEADY_GOOD` (20)
      - apply multipliers:
        - `M_pace(pace)`
        - `M_terrain(region)` (mountains 0.5)
        - `M_oxen(effective_oxen)` (if <4, multiply by effective_oxen/4; sick ox weight 0.5)
        - `M_party_sick(sick_count)` (HEALTH-008)
        - `M_snow(snow_depth)` (monotone 0..1)
      - apply `M_random_adjust(events_today)` (may be negative or hard-stop)
    - Ensure `miles_today = 0` on hard-stops and delayed days.
  - Spec refs:
    - Systems spec §8.1–§8.4 (base and multipliers), policy table values for base/mults.
  - Acceptance criteria:
    - Under OTDeluxe90sPolicy, the travel model does not depend on Dystrail’s current `TravelConfig` multipliers unless explicitly mapped.

- [x] TRAVEL-008 (P1) Implement terrain classification per node and use it in travel speed
  - Requirement:
    - Trail nodes (or segments) must carry a terrain classification sufficient to apply `TERRAIN_MULT[mountains]=0.5`.
  - Spec refs:
    - Systems spec §8.2 (terrain multiplier) and TrailGraph node list (§8.5).
  - Acceptance criteria:
    - The terrain multiplier is applied only where appropriate, deterministically.

- [ ] TRAVEL-009 (P0) Implement `ComputeMilesToday` as an explicit phase (distance fields + progress update + nav hard-stops)
  - Requirement:
    - ComputeMilesToday must:
      - compute `distance_today_raw` and `distance_today` for the day using the active policy model
      - apply navigation hard-stop events (lost/wrong/impassable/snowbound) using `rng.events()` with fixed draw order
      - apply policy-defined multi-day delays for navigation hard-stops (`delay_days_remaining`)
      - if travel occurs, update progress counters (`miles_traveled` and any “actual miles” field) and emit a `TravelProgress`-style event
    - Deluxe hard-stop rule (parity-critical):
      - if any hard-stop applies (boss gate, crossing pending, travel_blocked, navigation hard-stop), then `computed_miles_today = 0` for that day.
  - Spec refs:
    - Systems spec §13.7 (distance semantics + hard-stop/max leak warning), §14 phase 10; Kernel pseudocode “ComputeMilesToday (rng.travel + rng.events)”.
  - Acceptance criteria:
    - A day with a navigation hard-stop cannot update `miles_traveled` and cannot leak miles via `distance_today_raw`.
    - Draw ordering is documented and tested (RNG-006 + TEST-002).
  - Progress:
    - [x] TRAVEL-009A Defer OTDeluxe pace/rations + miles computation into the travel flow after block checks.

- [x] TRAVEL-010 (P1) Implement `TravelWearTick` as an explicit phase driven by “actual travel miles”
  - Requirement:
    - TravelWearTick must:
      - apply wear only after ComputeMilesToday has determined the day’s actual miles
      - apply 0 wear on any NonTravel day or hard-stopped day
      - be policy-gated so OTDeluxe90sPolicy can opt out of Dystrail-only wear systems if they violate parity
  - Spec refs:
    - Systems spec §14 phase 11 and §14.1 ownership boundaries; Journey diff “TravelWearTick is post-travel”.
  - Acceptance criteria:
    - Wear cannot be applied in StartOfDay or before travel distance is known.

---

## 11) Encounters + Random Events

- [x] ENCOUNTER-001 (P0) Derive encounter chance exactly once per day from a single function
  - Requirement:
    - Compute:
      - `encounter_base(region, mode, policy)`
      - plus deltas: pace, weather effects, exec orders, strain
      - minus cooldown penalties
      - clamp to policy cap
    - Roll once per day (respecting caps).
  - Spec refs:
    - Systems spec §13.6 and §14 phase 9; Journey diff “single-source and policy-driven”.
  - Acceptance criteria:
    - Weather does not mutate encounter chance directly outside the derivation function.
    - UI helpers do not “precompute” encounter chance by mutating state.

- [x] ENCOUNTER-002 (P0) Separate “navigation hard-stop events” from “non-navigation random events”
  - Requirement:
    - Navigation events that can hard-stop travel occur in ComputeMilesToday.
    - Non-navigation events occur in RandomEventTick and must not hard-stop travel (unless explicitly defined as hard-stop family).
  - Spec refs:
    - Systems spec §14 phases 10 vs 14; Kernel pseudocode separation.
  - Acceptance criteria:
    - Event taxonomy is explicit and prevents accidental hard-stops in the wrong phase.

- [x] ENCOUNTER-003 (P1) Implement event pools and context multipliers as data-driven tables
  - Requirement:
    - Random event selection must be:
      - base weights (data)
      - multiplied by circumstance function `F(context)` (policy/config)
  - Spec refs:
    - Systems spec §8.1–§8.2 and §9.4 (telemetry).
  - Acceptance criteria:
    - Changing event weights does not require code changes.

- [x] ENCOUNTER-004 (P0) Ensure Deluxe random-event families are covered (by events/encounters/vehicle/weather/exec orders)
  - Requirement:
    - The engine’s combined “event surface area” MUST cover the Deluxe families listed in the systems spec:
      - weather catastrophes (blizzard/hail/thunderstorm/fog/strong winds)
      - resource shortages (bad water/no water/no grass)
      - navigation (lost/wrong/rough/impassable/snowbound)
      - party incidents (lost member, snakebite)
      - oxen incidents (ox wandered off, ox sickness/death)
      - resource changes (abandoned wagon, thief, wild fruit, mutual-aid help find food, gravesite, fire)
      - wagon part breaks (repair/replace/unrepairable)
    - Each family must have:
      - a stable mechanical event kind/id
      - a deterministic payload (time loss, item loss, health effects, etc.)
      - a satire-safe presentation key.
  - Spec refs:
    - Systems spec §10 (random events list + mapping note).
  - Acceptance criteria:
    - A coverage test can enumerate the event catalog and confirm every required family is represented.

- [x] ENCOUNTER-005 (P0) Implement `RandomEventTick` (phase 14) as the sole applicator of non-navigation random events
  - Requirement:
    - Implement a distinct RandomEventTick phase that:
      - selects non-navigation events from the policy-defined pool
      - applies their deterministic effects
      - emits structured events + decision telemetry
    - RandomEventTick MUST NOT:
      - select weather
      - alter encounter selection state
      - hard-stop travel (unless the chosen event is explicitly marked as a hard-stop family and the spec allows it)
    - RandomEventTick runs only on days where the day has not already terminated early due to a blocking gate/intent:
      - Boss gate (StoppedNeedsChoice)
      - Ferry wait day
      - Rest/Trade/Hunt intents (non-travel day-atomic)
      - Route-variant choice pending (Sublette Cutoff / Dalles shortcut)
      - Crossing choice pending (StoppedNeedsChoice)
      - The Dalles final-route choice pending (StoppedNeedsChoice)
  - Spec refs:
    - Systems spec §14 phase 14; §14.1 ownership table; §15 invariants; Kernel pseudocode “Random events (non-navigation)”.
  - Current code notes (must change for parity):
    - Today, the “random events surface” is spread across encounters, vehicle, weather, and misc helpers; parity requires an explicit RandomEventTick for OTDeluxe90sPolicy.
  - Touchpoints (concrete):
    - `dystrail-game/src/state.rs::GameState::process_encounter_flow` (must not also be the RandomEventTick in parity mode)
    - `dystrail-game/src/encounters/*` (event pools / selection)
  - Acceptance criteria:
    - Non-navigation event selection and application is centralized and testable as a standalone tick.

---

## 12) TrailGraph + Deluxe Mile Markers + Route Variants

- [x] TRAIL-001 (P0) Implement `TrailGraph` and derive `current_node_index` from mile markers
  - Requirement:
    - Progression must be derived from `miles_traveled` vs the active mile-marker table, not ad-hoc milestone checks.
    - Active lists contain `0` sentinels for skipped nodes and MUST treat `0` as “node absent”.
    - End-of-trail arrival threshold must be derived from the active route-variant table (final non-zero mile marker),
      not hard-coded to a single value.
    - Oregon City strings are presentation-only and must not create an extra simulation node or miles beyond Willamette Valley.
  - Spec refs:
    - Kernel pseudocode `TrailGraph` + derivations; Systems spec §8.5 and explicit MUSTs at ~707/724/743.
  - Acceptance criteria:
    - Node arrival and store availability are functions of node index.

- [x] TRAIL-002 (P0) Implement route variants and ensure branching removes skipped nodes deterministically
  - Requirement:
    - Support these route variants (Deluxe EXE extracted):
      - main
      - Sublette Cutoff (skips Fort Bridger)
      - Dalles shortcut (skips Fort Walla Walla)
      - Sublette + Dalles combined (skips both)
    - When a choice is taken, the active mile-marker list switches for the remainder of the run.
  - Spec refs:
    - Systems spec §8.5; Kernel pseudocode “mile_markers_by_route_variant”.
  - Acceptance criteria:
    - Derived node indices remain monotone and skip absent nodes.

- [x] TRAIL-003 (P0) Implement The Dalles gate as a mandatory hard stop at node 16
  - Requirement:
    - At node 16 (The Dalles), the simulation MUST block travel beyond until a final route option is resolved.
    - Both options MUST be day-advancing, deterministic subflows emitting events.
    - Policy MUST define costs/time/outcomes for:
      - rafting
      - Barlow Toll Road
  - Spec refs:
    - Systems spec explicit MUST at ~852–855; Kernel pseudocode notes “This gate exists at node 16… MUST block…”.
  - Acceptance criteria:
    - The game cannot silently “pass” The Dalles; UI must prompt and state must persist the unresolved choice.

- [x] TRAIL-004 (P0) Implement the route-variant branch-choice prompts at the correct nodes (South Pass, Blue Mountains)
  - Requirement:
    - The route-variant choices are not abstract toggles; they must be offered at the correct branch points:
      - Sublette Cutoff prompt at South Pass (node index 7 on the main list)
      - “Shortcut to The Dalles” prompt at Blue Mountains (node index 14 on the main list)
    - When a choice is taken:
      - update `route_variant` deterministically for the remainder of the run
      - switch to the corresponding mile-marker table (which includes `0` sentinels for skipped nodes)
      - ensure node-triggered systems derive from the new route variant immediately (no stale indices), including:
        - store availability
        - store multiplier schedule
        - arrival triggers
        - end-of-trail arrival threshold
    - UI must present “stay on main route” vs “take shortcut” as an explicit choice; the simulation must not auto-select.
  - Spec refs:
    - Systems spec §8.5.2–§8.5.4 (branch points + parity contracts); Kernel pseudocode “route_variant selects active mile-marker list”.
  - Acceptance criteria:
    - Taking a shortcut cannot result in visiting skipped nodes (no store, no arrival trigger, no multiplier stage) because those indices are `0` and treated as absent.

---

## 13) Crossings (Dystrail crossings + OT river crossings)

- [x] CROSSING-001 (P0) Make CrossingTick a stop-and-choose gate (no auto-resolution in parity mode)
  - Requirement:
    - If a crossing is reached, travel must stop and the day must end in a “needs choice” UI state.
    - Miles must not advance beyond the crossing until choice is resolved.
  - Spec refs:
    - Systems spec §14 phase 13; Kernel pseudocode “CrossingChoiceNeeded”.
  - Current code notes (must change for parity):
    - `dystrail-game/src/state.rs::GameState::handle_crossing_event` currently auto-resolves a crossing outcome via RNG and applies it immediately.
    - Parity requires persisting a pending crossing choice and returning control to the UI without rerolling.
  - Touchpoints (concrete):
    - `dystrail-game/src/state.rs::GameState::handle_crossing_event`
    - `dystrail-game/src/crossings/resolver.rs` (current Dystrail crossing resolver)
  - Acceptance criteria:
    - UI has an explicit crossing phase; engine preserves pending crossing state.

- [x] CROSSING-002 (P0) Implement OT river crossings under OTDeluxe90sPolicy (ford/caulk-float/ferry/guide)
  - Requirement:
    - OTDeluxe rivers and special endgame river:
      - River crossings modeled: Kansas, Big Blue, Green, Snake
      - Columbia is handled separately as “The Dalles” final-route choice (rafting vs Barlow), not as a standard crossing
    - Provide OTDeluxe crossing options and constraints:
      - Ferry:
        - only where historically exists (policy defines which river nodes offer ferry)
        - cost `$FERRY_COST_CENTS = 500`
        - availability gate: `depth >= FERRY_MIN_DEPTH_FT = 2.5`
        - wait-days: `FERRY_WAIT_DAYS = 0..6` (policy-defined distribution; default uniform)
        - accident probability in `0.0..0.10` based on swiftness (function policy-defined)
      - Caulk/float:
        - mechanical minimum: `depth >= CAULK_FLOAT_MIN_DEPTH_FT = 1.5`
        - cost: `CROSSING_COST_DAYS` (default 1)
        - risk depends on depth+swiftness
        - Deluxe help guidance (non-mechanical): recommend attempting only when `depth > 2.5` (`CAULK_FLOAT_HELP_RECOMMENDED_MIN_DEPTH_FT`)
      - Ford:
        - cost: `CROSSING_COST_DAYS` (default 1)
        - wet goods if `2.5..3.0` (+`DRYING_COST_DAYS = 1`)
        - swamps `> 3.0` with losses scaling by depth
        - stuck/overturn modifiers by bed type (muddy/rocky)
      - Guide:
        - Snake only
        - costs `GUIDE_COST_CLOTHES = 3` sets of clothing
        - risk × `GUIDE_RISK_MULT = 0.20`
        - loss magnitude reduced (`GUIDE_LOSS_MULT`, default 0.50)
    - Outcome families MUST include:
      - safe, stuck_in_mud, supplies_wet, tipped, sank, drownings
  - Spec refs:
    - Systems spec §9 and explicit MUST at ~1386; Kernel pseudocode crossing resolver notes.
  - Acceptance criteria:
    - Crossing outcomes are policy-driven and emit structured events.
    - Ferry wait days are modeled as day-advancing non-travel days that still run root-cause ticks.

- [x] CROSSING-003 (P1) Keep existing Dystrail crossing model under DystrailLegacyPolicy
  - Requirement:
    - Maintain current checkpoint/bridge-out system as a separate crossing model.
    - Ensure it does not contaminate OTDeluxe90sPolicy behavior (no shared constants, no auto-bribe in parity mode).
  - Touchpoints (concrete):
    - `dystrail-game/src/state.rs::GameState::handle_crossing_event`
    - `dystrail-game/src/crossings/*` (resolver, types, config)
    - `dystrail-web/static/assets/data/crossings.json` (if used by the Dystrail model)
  - Acceptance criteria:
    - Switching policies switches crossing model deterministically.

---

## 14) Store / Economy / Capacity (OTDeluxe90s)

- [x] STORE-001 (P0) Implement OTDeluxe store inventory and pricing model (cents, per-node multipliers)
  - Requirement:
    - OTDeluxe store sells itemized goods and uses:
      - base prices in cents
      - per-node multiplier table `STORE_PRICE_MULT_PCT_BY_NODE` (normative)
    - Store availability must be restricted to the policy-defined store nodes:
      - includes the start-of-run store (e.g., “Matt's”) and the fort stops listed in `STORE_NODE_INDICES`
        - OTDeluxe90sPolicy normative list: `STORE_NODE_INDICES = [0, 3, 5, 8, 11, 13, 15]`
      - store availability must be derived from node index, not ad-hoc “is_fort” checks
      - if the active route variant skips a store node (0 sentinel for Fort Bridger / Fort Walla Walla), that store must be treated as absent (no store stop, no purchase)
    - Pricing formula must match the extracted Deluxe rule:
      - `price_cents(item, node_index) = STORE_BASE_PRICE_CENTS[item] * STORE_PRICE_MULT_PCT_BY_NODE[node_index] / 100`
      - Money math must be integer and deterministic (avoid float rounding).
    - Handle the extracted multiplier table shape explicitly:
      - `STORE_PRICE_MULT_PCT_BY_NODE` has 19 entries in the extraction; mapping the first 18 entries to node indices 0..17 is treated as normative.
      - The trailing entry (still 250%) must have a documented meaning (e.g., post-arrival stage) and must not cause out-of-bounds indexing.
  - Spec refs:
    - Systems spec §10 and policy table; explicit MUST at ~958 (caps must match).
  - Acceptance criteria:
    - Price calculation is deterministic and uses the policy multiplier for the current node.

- [x] STORE-002 (P0) Enforce OTDeluxe store max-buy caps and capacity model
  - Requirement:
    - Enforce per-item caps (OTDeluxe default):
      - oxen ≤ 20
      - ammo_boxes ≤ 50
      - clothes_sets ≤ 99
      - each spare type ≤ 3
      - food_lbs ≤ 2000
    - Default wagon capacity model is per-item caps only (no total-weight cap unless later proven).
  - Spec refs:
    - Systems spec policy table `STORE_MAX_BUY[...]`, `WAGON_CAPACITY_MODEL = per_item_caps`.
  - Acceptance criteria:
    - Purchase flows reject quantities above caps deterministically and emit appropriate events/log keys.

- [x] STORE-003 (P1) Align “money unit” and state fields (`cash_cents` vs existing `budget_cents`)
  - Requirement:
    - OTDeluxe economy uses `cash_cents` for store and score.
    - Engine currently has `budget_cents`; implement a clear mapping or introduce `cash_cents` for OTDeluxe policy.
  - Spec refs:
    - Systems spec §3.1 (cash_cents), §12.4 scoring formula.
  - Acceptance criteria:
    - OTDeluxe score uses `cash_cents / 500` as specified, not a Dystrail-specific budget abstraction.

- [x] STORE-004 (P0) Implement ammo-box purchase unit and conversion to bullets
  - Requirement:
    - Store purchases ammo as `ammo_boxes`.
    - `bullets = ammo_boxes * BULLETS_PER_BOX`, where `BULLETS_PER_BOX = 20` (policy-defined, Deluxe stated).
    - Bullets are consumed by hunting and used in scoring (`floor(bullets / 50)`).
  - Spec refs:
    - Systems spec §11 (store purchase units), policy table (`BULLETS_PER_BOX`, ammo prices).
  - Acceptance criteria:
    - Buying ammo changes bullets deterministically; hunting gating “no bullets” works.

- [x] STORE-005 (P2) Surface Deluxe store help recommendations as non-binding UI copy (parity flavor)
  - Requirement:
    - Display Deluxe-authored guidance (non-mechanical) under OTDeluxe90sPolicy:
      - at least six oxen
      - ≥200 lbs of food per party member
      - ≥2 sets of clothes per party member
      - spare parts recommendation
  - Spec refs:
    - Systems spec §11 (help recommendations).
  - Acceptance criteria:
    - This copy does not change mechanics (no hidden modifiers).

---

## 15) Scoring (OTDeluxe) + Endings

- [x] ENDGAME-001 (P0) Implement `EndgameTick` as a bounded phase (no core-physics mutation)
  - Requirement:
    - Implement the kernel’s EndgameTick (phase 12) as a *consumer* of the day’s resolved outcomes:
      - inputs: `computed_miles_today`, whether a breakdown started today, current progress, and policy toggles
      - outputs: endgame/boss readiness scaling, victory triggers, and any endgame-only derived state
    - EndgameTick MUST NOT:
      - change weather
      - change supplies burn
      - change health/affliction state for the same day
      - change travel distance for the same day
    - Under OTDeluxe90sPolicy:
      - endgame/victory is governed by TrailGraph end-of-trail arrival (Willamette Valley on the active route variant)
      - Dystrail boss/endgame scaling must be disabled unless explicitly opted in as a separate campaign rule-set.
  - Spec refs:
    - Systems spec §14 phase 12 and §14.1 ownership boundary; §8.5 (end-of-trail threshold); Journey diff “EndgameTick uses computed miles”.
  - Touchpoints (concrete):
    - `dystrail-game/src/endgame.rs::run_endgame_controller` (must be called only from EndgameTick under the kernel)
  - Acceptance criteria:
    - EndgameTick can be removed/disabled without changing core day physics (it is not entangled with weather/health/travel).

- [x] RECORD-001 (P0) Implement `RecordDay + TerminalChecks + EndOfDay` as the finalizing phase for every tick
  - Requirement:
    - The final kernel phase must:
      - finalize the `DayRecord` (kind, miles, tags, and audit snapshot)
      - emit/log derived output from events (but logs are not causal)
      - perform terminal checks (death/victory) and set the day outcome `ended` flag deterministically
      - advance day counters / mark `did_end_of_day` so the next tick cannot double-apply a day
    - Every early-return path (boss gate, forced wait day, rest/trade/hunt, travel blocked, crossing pending, Dalles pending, encounter selected) MUST still:
      - finalize a day record (or explicitly mark “day not consumed” if that’s the chosen semantics for the gate)
      - return a `DayOutcome` with `events[]` explaining the reason.
  - Spec refs:
    - Systems spec §1.1 (DayRecord), §14 phase 15, §15 invariants; Journey diff “RecordDay + Terminal”.
  - Touchpoints (concrete):
    - `dystrail-game/src/state.rs::GameState::record_travel_day`
    - `dystrail-game/src/state.rs::GameState::failure_log_key`
    - `dystrail-game/src/state.rs::GameState::end_of_day`
  - Acceptance criteria:
    - There is no code path that advances RNG/day physics without producing a DayRecord for that day.

- [x] SCORE-001 (P0) Implement OTDeluxe scoring formula under OTDeluxe90sPolicy
  - Requirement:
    - Implement the exact points and divisors:
      - wagon 50, ox 4, spare parts 2, clothes 2, bullets/50, food/25, cash/5
    - Multiply by occupation bonus multiplier.
    - Per-person points use `SCORE_POINTS_PER_PERSON_BY_HEALTH` (Good=500, others default 0).
  - Spec refs:
    - Systems spec §12 and policy table; explicit MUST at ~1444.
  - Acceptance criteria:
    - Score breakdown is reproducible and matches spec for a known test state.

- [ ] SCORE-002 (P1) Define and document how OTDeluxe victory/end state integrates with Dystrail boss gate
  - Requirement:
    - OTDeluxe progression ends at Willamette Valley; Dystrail currently has a boss gate.
    - Policy selection must determine which “end condition” is authoritative for the run.
  - Spec refs:
    - Systems spec §13.3 and trail sections; Journey diff mentions boss gate ordering.
  - Acceptance criteria:
    - Under OTDeluxe90sPolicy, end-of-trail behavior is consistent with TrailGraph and scoring.

---

## 16) UI & UX Requirements (to support phase gates; satire remains presentation-only)

- [x] UI-001 (P0) Add UI phases for “needs choice” states (Crossing, The Dalles, Store, Trade, Hunt)
  - Requirement:
    - Any kernel outcome that blocks travel pending a choice must have a UI surface.
    - UI must not advance the day without providing the required choice back to the kernel.
    - UI surfaces must exist for all parity-critical gates/choices, including:
      - Boss gate (Dystrail-only; disabled under OTDeluxe90sPolicy unless explicitly opted in)
      - River crossing choice (ford/caulk-float/ferry/guide) under OTDeluxe90sPolicy
      - Route-variant prompts (Sublette Cutoff at South Pass; Dalles shortcut at Blue Mountains)
      - The Dalles final-route choice (raft vs Barlow)
  - Spec refs:
    - Kernel pseudocode `StoppedNeedsChoice` outcomes; Systems spec phase gating rules.
  - Acceptance criteria:
    - The player can resolve crossings and route choices without implicit auto-resolve.

- [x] UI-002 (P1) Render logs from events (not vice versa)
  - Requirement:
    - UI rendering consumes `events[]` and translates them to localized satire copy keys.
  - Spec refs:
    - Systems spec §4.2 and satire contract §2.1.
  - Acceptance criteria:
    - UI does not need engine “log_key” as the only output; it can render multiple events per day.

---

## 17) Tests, Determinism, and Migration

- [x] TEST-001 (P0) Add phase-order tests for kernel pipeline
  - Requirement:
    - Tests must assert the kernel executes phases in the exact order required by §14 and that phases mutate only their owned state slices.
  - Spec refs:
    - Systems spec §14 and §14.1.
  - Acceptance criteria:
    - A regression test fails if, e.g., WeatherTick is moved after HealthTick.

- [x] TEST-002 (P0) Add RNG-scope tests (streams + draw counts)
  - Requirement:
    - Tests must detect if weather consumes travel RNG, etc.
  - Spec refs:
    - Systems spec §4.1 and §15; RNG checklist above.
  - Acceptance criteria:
    - For a fixed seed/day context, stream draw counts are stable and documented.

- [x] TEST-003 (P0) Update/replace deterministic digest baselines knowingly
  - Requirement:
    - Existing “journey digest baseline” tests will drift after refactor; update them only after validating parity intent.
  - Touchpoints:
    - `dystrail-tester/src/logic/playability.rs` deterministic digest.
  - Acceptance criteria:
    - New baselines are checked in only with explicit sign-off that the kernel refactor is correct.

- [ ] LOCK-001 (P1) Add a reproducible `OREGON.EXE` extraction pipeline for remaining Deluxe-unknown parameters
  - Requirement:
    - The systems spec intentionally keeps some knobs “policy-defined” pending extraction/fit.
    - To reduce long-term ambiguity, implement an explicit extraction pipeline that:
      - reads a pinned `OREGON.EXE` binary (hash-verified)
      - extracts remaining tables/constants when possible
      - writes a machine-readable artifact (e.g., `otdeluxe90s_extracted.json`) consumed by `OTDeluxe90sPolicy`
    - Target extraction list (parity-critical, currently not confirmed by EXE in the spec):
      - health constants if present in code path (`HEALTH_RECOVERY_BASELINE`, label ranges, death threshold)
      - affliction probability curve (health → 0..0.40)
      - ferry wait-days distribution implementation (0..6 mapping)
      - ferry accident outcome families/weights (by swiftness)
      - crossing per-method outcome weights (ford/float/ferry), and depth/swiftness modifiers if table-driven
      - occupation numeric perks (doctor fatality mult, repair success, farmer mobility mult) if present as tables
      - death-imminent grace-days countdown semantics (if encoded as constants)
      - climate/station tables and precip thresholds (optional; only if we choose to implement OTDeluxeStationsWeather)
      - scoring tiers for Fair/Poor/Very Poor arrivals (if present)
    - This pipeline must be versioned and repeatable so policy updates are auditable and not hand-edited.
  - Spec refs:
    - Systems spec §0 sources, §4.3.1 extracted constants, §16 “still open” items.
  - Acceptance criteria:
    - The repo can regenerate the extracted policy artifact from a known EXE (or a fixture) and compare it to the committed artifact.
    - Any extracted value that differs from the spec triggers a deliberate review/update (no silent drift).

- [ ] LOCK-002 (P1) Add an empirical sampling harness to fit remaining distributions (when EXE extraction is infeasible)
  - Requirement:
    - Some behaviors may be data-driven at runtime or difficult to extract statically; for those, implement a sampling harness that:
      - runs controlled Deluxe scenarios (save-state or deterministic setup)
      - records outcomes into histograms/tables
      - produces fitted policy parameters (with confidence intervals when relevant)
    - Target fit list (from systems spec §16 “still open” items):
      - affliction curve shape (health → p_affliction)
      - ferry wait-days histogram
      - crossing outcome tables by method at fixed depth/swiftness
      - navigation delay distributions for lost/wrong/impassable/snowbound
      - occupation perk numerics (Monte Carlo comparisons by profession)
      - death-imminent grace-days and reset behavior
      - scoring tiers for non-Good arrivals
    - The harness must clearly separate:
      - Deluxe-observed facts
      - fitted parameters
      - assumed defaults (when neither extraction nor fit is available)
  - Spec refs:
    - Systems spec §16 lock paths; Journey diff “reduce policy-defined unknowns”.
  - Acceptance criteria:
    - Running the harness produces reproducible artifacts under pinned inputs (seed/save-state + EXE hash).
    - Fitted policy parameters can be updated without modifying kernel code.

- [x] MIGRATION-001 (P0) Add state version bump + save migration for new fields
  - Requirement:
    - New state fields (health_general, general_strain, accumulators, route variant, pending intents, etc.) must be migrated from old saves.
  - Acceptance criteria:
    - Old saves load without panic; missing fields are defaulted deterministically.

---

## 18) “No Contradictions” Audit Checklist (run before coding starts)

Use this section as a pre-implementation sanity pass. Every item must be true in the final codebase.

- [x] AUDIT-001 `StartOfDay` does not apply physics (only resets/cooldowns) and does not consume RNG.
- [x] AUDIT-002 Weather selection uses only `rng.weather()`; afflictions use only `rng.health()`.
- [x] AUDIT-003 Supplies burn runs exactly once per day and is always before HealthTick.
- [x] AUDIT-004 Encounter chance is derived once per day from a single function; weather does not mutate it ad hoc.
- [x] AUDIT-005 OTDeluxe hard-stops always produce `computed_miles_today = 0`; no “delay travel credit” leaks.
- [ ] AUDIT-006 All “policy-defined” values are explicit in OTDeluxe90sPolicy config and not hard-coded.
- [x] AUDIT-007 TrailGraph uses Deluxe mile markers + route variants; `0` sentinels are skipped in derivations.
- [x] AUDIT-008 The Dalles gate blocks travel beyond node 16 until resolved.
- [x] AUDIT-009 Crossing choices are interactive under OTDeluxe90sPolicy; outcomes include the required families.
- [x] AUDIT-010 Scoring under OTDeluxe90sPolicy matches the spec exactly (including occupation multiplier).
- [ ] AUDIT-011 Satire only affects presentation; it never changes RNG/phase order/numbers.

- [x] AUDIT-012 OTDeluxe pace + rations affect both consumption and health (not one or the other).
- [x] AUDIT-013 Party/oxen state exists and feeds travel viability, hunting carry cap, and scoring.
- [x] AUDIT-014 Deluxe random-event families are all represented by the engine’s event catalog.
- [x] AUDIT-015 Store availability is derived from `STORE_NODE_INDICES` (including the start store) and respects skipped nodes via `0` sentinels.
- [x] AUDIT-016 Route-variant prompts occur at South Pass and Blue Mountains; taking a shortcut cannot visit skipped nodes or apply their store multipliers.

---

## 19) Open Questions / Unresolvable Contradictions (needs your answers)

As of this checklist draft, there are **no spec-internal contradictions that require a decision** beyond what’s already locked.

However, there are two implementation-scope clarifications that must be made explicit during coding (they do not change the spec, but they constrain architecture):

1) **How OTDeluxe90sPolicy coexists with the existing Dystrail campaign loop**
   - The spec requires OTDeluxe TrailGraph, stores, crossings, and scoring under OTDeluxe90sPolicy.
   - The current Dystrail “boss gate” and Dystrail score system must therefore be gated to non-OT policies.
   - Implementation must choose whether OTDeluxe90sPolicy is:
     - (A) a distinct “OTDeluxe campaign” mode, or
     - (B) the default mechanics for the main campaign.
   - The checklist assumes (A) because it avoids breaking existing Dystrail runs, but either can satisfy the spec if policy selection is explicit.

2) **Mapping OT itemized inventory into Dystrail UI affordances**
   - OTDeluxe requires itemized oxen/food/bullets/clothes/spares; Dystrail currently uses umbrella supplies + tags/spares.
   - Implementation must decide whether the UI shows OT inventory directly (recommended for parity) or shows a Dystrail-styled abstraction that still preserves the exact mechanics and score components.

3) **Route-variant prompt day semantics (South Pass / Blue Mountains)**
   - The spec defines *where* the prompts occur and how they change mile-marker tables, but does not state whether choosing a route variant:
     - (A) is purely an immediate choice at arrival with no special day-record implications, or
     - (B) behaves like other “StoppedNeedsChoice” gates (the day ends at the landmark; choice is resolved in UI; next tick continues with the selected route variant).
   - The checklist currently assumes (B) for determinism symmetry with crossings/Dalles, but if Deluxe UX implies (A), we should lock that.

4) **Boss gate “consumes a day” semantics (DystrailLegacyPolicy only)**
   - OTDeluxe90sPolicy disables boss gating unless explicitly opted in, but DystrailLegacyPolicy still needs a clear rule:
     - Does boss gating pause the day (no day record finalized until the boss is resolved), or
     - does it record a NonTravel day and advance time?
   - This affects replay determinism and UX (re-rolling weather/supplies/health while “awaiting boss” is disallowed).

If you want any of these constrained more tightly (A vs B; UI representation choice), answer here and we’ll lock it as an additional implementation-binding decision.
