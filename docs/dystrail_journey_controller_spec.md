# Dystrail Journey Controller — Hyper‑Detailed Engineering Specification

**Document purpose:** Replace disparate travel/encounter special‑cases with a **single, policy‑driven Journey Controller** that reproduces the _Oregon Trail_ feel—**total distance ≈ 2,000 miles, duration 3–6 months (≈84–180 days), average pace 10–20 miles/day**—and remains tunable, deterministic, and testable across _Classic_ and _Deep_ policy families.

---

## Contents

1. [Scope & Goals](#scope--goals)
2. [Non‑Goals](#non-goals)
3. [Experience Targets (“Oregon‑Trail Feel”)](#experience-targets-oregontrail-feel)
4. [System Overview](#system-overview)
   - 4.1 [Single Journey Controller](#single-journey-controller)
   - 4.2 [Policy Stack](#policy-stack)
   - 4.3 [Determinism & RNG Streams](#determinism--rng-streams)
   - 4.4 [Telemetry & Tester Metrics](#telemetry--tester-metrics)
5. [Phase A — **Invariant Grounding** (Complete by end of phase: _stable math & day accounting_)](#phase-a--invariant-grounding)
   - A.1 [Mathematical Model](#a1-mathematical-model)
   - A.2 [Day Accounting Semantics](#a2-day-accounting-semantics)
   - A.3 [Unified Record API](#a3-unified-record-api)
   - A.4 [Unit Tests (Phase A)](#a4-unit-tests-phase-a)
6. [Phase B — **Deterministic RNG Architecture** (_no mixed outcomes; seed‑stable_)](#phase-b--deterministic-rng-architecture)
   - B.1 [RNG Streams & Derivation](#b1-rng-streams--derivation)
   - B.2 [Crossing Resolver Atom](#b2-crossing-resolver-atom)
   - B.3 [Encounters & Rotation](#b3-encounters--rotation)
   - B.4 [Unit & Property Tests (Phase B)](#b4-unit--property-tests-phase-b)
7. [Phase C — **Vehicle Wear & Breakdown Pipeline** (_Oregon‑feel pacing, no ad‑hoc saves_)](#phase-c--vehicle-wear--breakdown-pipeline)
   - C.1 [Wear Model](#c1-wear-model)
   - C.2 [Breakdown Probability](#c2-breakdown-probability)
   - C.3 [Field Repair & Costs](#c3-field-repair--costs)
   - C.4 [Endgame Grace (Policy‑Gated)](#c4-endgame-grace-policy-gated)
   - C.5 [Unit Tests (Phase C)](#c5-unit-tests-phase-c)
8. [Phase D — **Crossing Engine** (_atomic outcomes; tunable success/detour/fail bands_)](#phase-d--crossing-engine)
   - D.1 [Outcome Distribution by Policy](#d1-outcome-distribution-by-policy)
   - D.2 [Bribe & Permit Logic](#d2-bribe--permit-logic)
   - D.3 [Detour Day Costs & Partial Travel](#d3-detour-day-costs--partial-travel)
   - D.4 [Unit & Statistical Tests (Phase D)](#d4-unit--statistical-tests-phase-d)
9. [Phase E — **Endgame Controller** (_miles 1,850+; seal the 2,000 mi experience_)](#phase-e--endgame-controller)
   - E.1 [Activation & Deactivation](#e1-activation--deactivation)
   - E.2 [Pacing, Stops, and Wear Suppression](#e2-pacing-stops-and-wear-suppression)
   - E.3 [Travel‑Ratio Guardrails](#e3-travel-ratio-guardrails)
   - E.4 [Unit & Multi‑Run Tests (Phase E)](#e4-unit--multi-run-tests-phase-e)
10. [Phase F — **Policy Catalog** (_Classic vs Deep; Strategy‑specific tuning_)](#phase-f--policy-catalog)
    - F.1 [Common Policy Shape](#f1-common-policy-shape)
    - F.2 [Classic Defaults](#f2-classic-defaults)
    - F.3 [Deep Defaults](#f3-deep-defaults)
    - F.4 [Strategy Overlays](#f4-strategy-overlays)
    - F.5 [Unit Tests (Phase F)](#f5-unit-tests-phase-f)
11. [Phase G — **Health/Sanity/Supplies Tick** (_pace & diet driven; exec orders & weather_)](#phase-g--healthsanitysupplies-tick)
    - G.1 [Daily Tick](#g1-daily-tick)
    - G.2 [Weather & Exec Orders](#g2-weather--exec-orders)
    - G.3 [Camp Actions](#g3-camp-actions)
    - G.4 [Unit Tests (Phase G)](#g4-unit-tests-phase-g)
12. [Phase H — **Telemetry, Metrics, and Tester**](#phase-h--telemetry-metrics-and-tester)
    - H.1 [Derived Metrics](#h1-derived-metrics)
    - H.2 [Acceptance Gates](#h2-acceptance-gates)
    - H.3 [Unit & Integration Tests (Phase H)](#h3-unit--integration-tests-phase-h)
13. [Phase I — **Data Shapes & Migration**](#phase-i--data-shapes--migration)
    - I.1 [Configuration Files](#i1-configuration-files)
    - I.2 [Serialization & Back‑compat](#i2-serialization--back-compat)
    - I.3 [Unit Tests (Phase I)](#i3-unit-tests-phase-i)
14. [Performance & Determinism Guarantees](#performance--determinism-guarantees)
15. [Appendix A — Math Details & Pseudocode](#appendix-a--math-details--pseudocode)
16. [Appendix B — Reference Defaults & UI ties](#appendix-b--reference-defaults--ui-ties)

---

## Scope & Goals

- **Single journey controller** governs daily loop: pace → miles → wear → breakdowns → crossings → encounters → consumption → logging.
- **Policy‑driven behavior**: _Classic_ and _Deep_ families with strategy overlays (Balanced, Aggressive, Conservative, ResourceManager, MonteCarlo).
- **Determinism**: identical seed ⇒ identical full trace; atomic crossings; stable RNG consumption.
- **No ad‑hoc branches**: knobs live in policy config; code path is uniform.
- **Tunability**: JSON/YAML config for all multipliers and thresholds.
- **Experience parity** with _Oregon Trail_:
  - Total **distance ~2,000 miles**;
  - **Duration 3–6 months**;
  - **Daily miles 10–20** (policy‑weighted);
  - Failure modes distributed across sanity/vehicle/encounters;
  - Crossings produce believable detours without RNG artifacts.

## Non‑Goals

- No UI/asset work beyond exposing new telemetry and policy name.
- No re‑theme or art changes (see asset inventory for future work, not in scope here).

## Experience Targets (“Oregon‑Trail Feel”)

Let total distance target **D\*** = 2,000 miles; target day window **T** in `[84, 180]` days; average miles/day **m̄** in `[10, 20]`.

Controllers and policies MUST produce aggregates across 1,000+ runs that satisfy:

- **Core distance / duration (all families & strategies)**

  - Mean distance: `1900 ≤ mean_miles ≤ 2100`
  - Mean duration: `84 ≤ mean_days ≤ 180`
  - Mean miles per day: `10 ≤ mean_mpd ≤ 20`
  - Travel ratio: `travel_ratio ≥ 0.90` for all non-experimental policies

- **Classic / Balanced — canonical Oregon Trail parity**

  - Boss reach rate (runs that reach boss): `0.30 ≤ boss_reach ≤ 0.50`
  - Boss win rate (runs that defeat boss): `0.20 ≤ boss_win ≤ 0.35`
  - Run survival (non–early-wipe endings of any type): `0.60 ≤ survival ≤ 0.80`
  - Failure mix: no single failure family (vehicle, sanity, exposure, crossings) exceeds `0.50` of all failures over large samples

- **Other Classic strategies (Aggressive, Conservative, ResourceManager, MonteCarlo)**

  - Share the same **distance/duration/mpd bands** as Classic/Balanced.
  - Aggressive: biased to **lower survival** and **more terminal crossings** than Balanced, but still with a meaningful path to victory (boss win often below Balanced band).
  - Conservative / ResourceManager: tilt toward **higher survival** and **slightly higher boss reach**, but boss win must not exceed ~`0.40` so that the game remains failure-prone.
  - MonteCarlo: same bands, but with higher per-run variance in distance, crossings, and failure causes.

- **Deep family (all strategies) — same bands, higher variance / weirdness**
  - Distance/duration/mpd bands are **the same** as Classic: the mean behavior must still orbit OT-style journeys.
  - Per-run variance is allowed to be **higher** (more “weird” runs), and tails may be heavier, but long-run means must still satisfy the bands above.
  - Deep may allow **slightly harsher crossings** or **stranger failure mixes**, but must not drift into a fundamentally different pacing model (e.g., short arcade-like runs or ultra-long slogs).

---

## System Overview

### Single Journey Controller

New module **`journey`** with public type:

```rust
pub struct JourneyController {
  policy: PolicyId,            // Classic | Deep
  strategy: StrategyId,        // Balanced | Aggressive | Conservative | ResourceManager | MonteCarlo
  cfg: JourneyCfg,             // resolved config (merged family + strategy overlay)
  rng: RngBundle,              // independent RNG streams
}
```

Core entry per day (uniform across policies):

```rust
pub fn tick_day(&mut self, gs: &mut GameState) -> DayOutcome;
```

### Policy Stack

- `JourneyCfg = FamilyCfg ⊕ StrategyOverlay` (overlay wins per‑field).
- Families: **Classic**, **Deep**.
- Strategy overlays: tweak pace bias, stop cadence, risk appetites, crossing priors.

### Determinism & RNG Streams

- Streams: `rng_travel`, `rng_breakdown`, `rng_encounter`, `rng_crossing`.
- Derived via HMAC‑SHA256(seed, domain_tag) → 64‑bit seeds.
- **Atomic crossing**: one stream consumption per crossing event, irrespective of outcome branch.

### Telemetry & Tester Metrics

- Metrics computed **only** from `Record::Day` events (see Phase A) to avoid drift.
- Acceptance gates reside in tester config; simulation never introspects gates.

---

## Phase A — **Invariant Grounding**

> **Completion definition:** day accounting is mathematically consistent; a single API records every day outcome; derived metrics equal sums of primitive counters.

### A.1 Mathematical Model (ASCII, implementation‑ready)

**Notation.** t = day index (0‑based). D(t) = cumulative miles at start of day t. kind(t) ∈ {Travel, Partial, NonTravel}. ρ_partial = partial‑day mileage ratio (policy‑tunable).

```text
# Miles per day (mpd)
mpd = clamp( mpd_base
             * pace_mult
             * weather_mult
             * exec_mult
             * health_mult
             * cargo_mult,
             mpd_min, mpd_max )

# Distance accumulation
if kind(t) == Travel:
    D(t+1) = D(t) + mpd
elif kind(t) == Partial:
    D(t+1) = D(t) + ρ_partial * mpd
else:  # NonTravel
    D(t+1) = D(t)

# Travel ratio over N days
travel_ratio = (count(Travel) + count(Partial)) / N
```

**Default** ρ_partial = 0.5 (policy‑tunable). **Acceptance:** travel_ratio ≥ R_min (default 0.90; per policy/strategy).

### A.2 Day Accounting Semantics

Introduce **`TravelDayKind`** with _exclusive_ values:

- `Travel`: full day of travel.
- `Partial`: limited travel due to detours/field repairs/EOs.
- `NonTravel`: camp, full repair, vote/boss day, terminal events.

All mileage credited to the run MUST come exclusively from DayRecord entries produced by record_travel_day.

### A.3 Unified Record API

```rust
pub enum TravelDayKind { Travel, Partial, NonTravel }

pub struct DayRecord {
  pub day_index: u16,
  pub kind: TravelDayKind,
  pub miles: f32,              // already partial‑adjusted
  pub tags: SmallVec<[Tag; 4]> // e.g., ["camp", "repair", "crossing_pass", "detour"]
}

pub fn record_travel_day(gs: &mut GameState, rec: DayRecord);
```

**Derived metrics** (`travel_days`, `partial_travel_days`, `non_travel_days`, `travel_ratio`, etc.) become **pure sums** over `DayRecord` history.

### A.4 Unit Tests (Phase A)

- **T‑A1**: `record_travel_day` idempotent accounting; sum(miles) == delta(D).
- **T‑A2**: `Partial` uses exactly \( \rho\_{\text{partial}} \) multiplier; fuzz \( \rho \in [0.25, 0.75] \).
- **T‑A3**: Mixed sequences reconstruct \( R \) exactly; property test with random sequences.
- **T‑A4**: Serialization round‑trip preserves `DayRecord` and derived metrics (no float drift beyond 1e‑6).

---

## Phase B — **Deterministic RNG Architecture**

> **Completion definition:** per‑domain RNG streams; crossing outcomes consume fixed draws; identical seeds ⇒ identical traces.

### B.1 RNG Streams & Derivation

```rust
pub struct RngBundle {
  travel: SmallRng,     // pace & weather jitter, mpd micro‑variance
  breakdown: SmallRng,  // wear events
  encounter: SmallRng,  // narrative/rotation
  crossing: SmallRng,   // atomic resolver
}
```

Seed derivation:

- `root = xxh64(user_seed)`
- `seed_travel = hash(root, "travel")`
- `seed_break = hash(root, "breakdown")`
- `seed_enc = hash(root, "encounter")`
- `seed_cross = hash(root, "crossing")`

### B.2 Crossing Resolver Atom

Single function consumes **exactly 1** draw block per crossing:

```rust
pub struct CrossingOutcome { kind: CrossingKind, days: u8, bribe_used: bool }
pub enum  CrossingKind { Pass, Detour(u8), Terminal }

pub fn resolve_crossing(cidx: u8, day: u16, ctx: &CrossingCtx, rng: &mut SmallRng) -> CrossingOutcome;
```

- Input seeds on `(policy_id, strategy_id, cidx, day)`; stream remains local to `rng_crossing`.
- No branching draws; map **one** sampled `u32` to three regions `[0, p_pass), [p_pass, p_pass+p_detour), else terminal]`.

### B.3 Encounters & Rotation

- Maintain rotation queue; consume exactly one `encounter` draw per day eligible for encounter; deterministic modulo queue mechanics.
- No re‑draws for rejected branches; rejection encoded as a tag in `DayRecord`.

### B.4 Unit & Property Tests (Phase B)

- **T‑B1**: Same seed ⇒ identical `DayRecord` timeline across 10k days.
- **T‑B2**: Crossing atom consumes fixed draws; instrument counter must equal number of crossings.
- **T‑B3**: Rotation queue deterministic under fixed policy; property: permutation invariants hold.
- **T‑B4**: Fuzz multiple seeds; no panics; output hash stable CRC across runs.

---

## Phase C — **Vehicle Wear & Breakdown Pipeline**

> **Completion definition:** wear increments and breakdowns use a single formula; field repair produces partial days; endgame grace handled by policy (no ad‑hoc saves).

### C.1 Wear Model

Let wear \( w*t \in [0,\infty) \). Daily increment: \[ \Delta w_t = \alpha_0 \cdot P*{pace} \cdot W\_{weather} \cdot \phi(D(t)) \]

- \( \alpha_0 \) base wear/unit day (policy).
- \( \phi(D) = 1 + \kappa \cdot \max(0, D - D\_{\text{comfort}})/400 \) (fatigue ramp).

### C.2 Breakdown Probability

Per day breakdown probability: \[ p*t = \min\left(1, \, p_0 \cdot (1 + \beta w_t)\cdot P*{pace}\cdot W\_{weather}\right) \]

Default anchors from `vehicle.json` (base breakdown chance, pace & weather factors, part weights, repair costs).

Part selection on breakdown uses weighted draw (`part_weights`).

### C.3 Field Repair & Costs

- Field repair converts the day to **`Partial`** with \( \rho\_{\text{partial}} \); increments `days_with_repair`.
- Costs: as in config (`repair_costs.use_spare_supplies`, etc.).
- Optional **mechanic_hook** (policy‑gated) adds full **`NonTravel`** day with higher fix certainty.

### C.4 Endgame Grace (Policy‑Gated)

- Activation distance \( D \ge D\_{\text{grace}} \) (default 1850).
- Options by policy flag:
  1. **Probabilistic suppression**: scale \( p_t \leftarrow (1-\gamma) p_t \).
  2. **Wear shave**: \( w_t \leftarrow \eta w_t \) post partial travel.
  3. **One‑time full reset**: once per run, set \( w_t \leftarrow 0 \) on first endgame breakdown (no ad‑hoc save elsewhere).

### C.5 Unit Tests (Phase C)

- **T‑C1**: Wear increases monotonically with pace & weather multipliers.
- **T‑C2**: Breakdown Bernoulli with base equals config; chi‑square within 3σ on 100k trials.
- **T‑C3**: Field repair causes `Partial` day and exact cost debits; asserts on ledger.
- **T‑C4**: Endgame grace switches exactly at `D_grace`; each option validated separately.

---

## Phase D — **Crossing Engine**

> **Completion definition:** crossings resolve to pass/detour/terminal from a single draw; permit/bribe alter priors; detours yield partials or additional `NonTravel` days per policy.

### D.1 Outcome Distribution by Policy

For a sampled \( u\in[0,1) \), thresholds:

- Pass if \( u < p\_{\text{pass}} \)
- Detour if \( p*{\text{pass}} \le u < p*{\text{pass}} + p*{\text{detour}} \) with detour days \( k\sim \text{Discrete}[k*{\min},k\_{\max}] \)
- Terminal otherwise

Targets (aggregated bands, tunable):

- **Classic**: terminal ≤ 12%, detour 1–3 days, pass remainder.
- **Deep**: terminal 12–16% (default), detour 1–4 days.

### D.2 Bribe & Permit Logic

- **Permit** sets \( p\_{\text{terminal}}=0 \) for eligible checkpoints (policy list; e.g., press pass).
- **Bribe** shifts mass: \( p*{\text{pass}}↑ \), \( p*{\text{terminal}}↓ \) with diminishing returns; success tracked for telemetry.
- Bands must meet tester warnings (e.g., bribe success ≥ 70% for certain scenarios) as configured in tester.

### D.3 Detour Day Costs & Partial Travel

- Detour consumes **`Partial`** day(s): each detour day calls `record_travel_day(Partial, \rho_{\text{partial}} \cdot m_t, tag=["detour"])`.
- Final day of a multi‑day detour may include crossing resolution (policy switch).

### D.4 Unit & Statistical Tests (Phase D)

- **T‑D1**: Resolver uses one draw per crossing; instrumented counter exact.
- **T‑D2**: Permit eliminates terminal branch for applicable crossings; property test across seeds.
- **T‑D3**: Bribe raises success and lowers terminal within configured delta; CI test on 100k samples.
- **T‑D4**: Aggregated fail rate stays within band per policy over 5k full simulations.

---

## Phase E — **Endgame Controller**

> **Completion definition:** from \( D \ge 1850 \) the controller ensures believable finish (≤30 days to go at 10–20 mpd) without violating travel‑ratio or breakdown gates.

### E.1 Activation & Deactivation

- Activate when \( D \ge D\_{\text{grace}} \) and no terminal lock.
- Deactivate at victory or terminal end.
- Single boolean in state: `endgame_active` (already present in traces; maintained here).

### E.2 Pacing, Stops, and Wear Suppression

- Bias \( m*t \) upward by factor \( b*{\text{end}} \in [1.02,1.10] \) to tighten finish window.
- Convert excessive full stops into **`Partial`** if 2‑in‑10 rolling cap exceeded (cap tunable per strategy).
- Apply **wear suppression** choice from Phase C.4.

### E.3 Travel‑Ratio Guardrails

Maintain \( R \ge R\_{\min} \) by:

1. Prefer `Partial` over `NonTravel` when forced interventions occur.
2. Shift non‑critical narrative events outside endgame (rotation deferral).

### E.4 Unit & Multi‑Run Tests (Phase E)

- **T‑E1**: Rolling stop cap converts the 3rd+ stop in any 10‑day window to `Partial`.
- **T‑E2**: Expected finish window: with default params and Clear weather, 150 mi remaining completes within 8–18 days on 95% runs.
- **T‑E3**: Over 1k runs, travel ratio never < policy \( R\_{\min} \).

---

## Phase F — **Policy Catalog**

> **Completion definition:** both families (_Classic_, _Deep_) and all strategies resolve to a single `JourneyCfg`; no code branches are policy‑specific.

### F.1 Common Policy Shape

```rust
#[derive(Deserialize)]
pub struct FamilyCfg {
  // Pace & miles
  pub mpd_base: f32,                // nominal miles/day before multipliers
  pub mpd_min:  f32, pub mpd_max: f32,
  pub partial_ratio: f32,           // ρ_partial

  // Wear & breakdowns
  pub wear_base: f32,               // α0
  pub wear_fatigue_k: f32,          // κ
  pub breakdown_base: f32,          // p0 (see vehicle.json)  // ref
  pub pace_factor: HashMap<Pace, f32>,   // steady/heated/blitz // ref
  pub weather_factor: HashMap<Weather, f32>, // Clear/Storm/... // ref

  // Crossings
  pub crossing: CrossingPolicy,     // priors, detour bands, permit rules, bribe deltas

  // Endgame
  pub endgame_distance: f32,
  pub endgame_bias: f32,
  pub endgame_stop_cap_rolling_10: u8,
  pub endgame_wear_mode: EndgameWearMode,

  // Acceptance guards (passed to tester)
  pub min_travel_ratio: f32,
  pub target_distance: f32,         // ~2000
  pub target_days_min: u16, pub target_days_max: u16,
}
```

### F.2 Classic Defaults

- `mpd_base ≈ 14.0`, `[mpd_min, mpd_max]=[8, 22]`
- Crossing terminal target ≤ 12% aggregated; detour 1–3 days.
- Endgame bias 1.04; stop cap 2‑in‑10.

### F.3 Deep Defaults

- Slightly higher stress and harsher crossings: terminal 12–16%; detour 1–4 days.
- Endgame wear suppression stronger to prevent late cascades.

### F.4 Strategy Overlays

- **Aggressive**: `pace_factor` bias to heated/blitz; higher wear; stricter stop cap.
- **Conservative**: slower pace; lower wear; higher permit/avoidance weighting.
- **ResourceManager**: cheaper repairs; higher chance to choose mechanic path.
- **MonteCarlo**: exploration‑friendly priors and more stable crossings.

### F.5 Unit Tests (Phase F)

- **T‑F1**: Overlay application is pure & associative; `Family ⊕ Overlay1 ⊕ Overlay2` deterministic.
- **T‑F2**: Each strategy yields `JourneyCfg` with required fields; no `None`.
- **T‑F3**: Sanity check: `mpd_min ≤ mpd_base ≤ mpd_max` for all resolved configs.

---

## Phase G — **Health/Sanity/Supplies Tick**

> **Completion definition:** daily tick uses a single formula; pace/diet/weather/EOs impact via multipliers; camps/repairs feed through `DayRecord` only.

### G.1 Daily Tick

Let HP, SAN, SUP be integer pools. For each day:

- Supplies: \( \Delta SUP = - s*0 \cdot P*{pace}\cdot \theta*{weather}\cdot \xi*{EO} \)
- Sanity: \( \Delta SAN = - q*0 \cdot P*{pace}\cdot \psi*{diet}\cdot \theta*{weather} \)
- Health: small decay unless `camp_rest` or medical encounter. All constants tunable in policy; diet and pace icons/UX not altered (see assets inventory).

### G.2 Weather & Exec Orders

Multipliers pulled from config; defaults for `weather_factor` exist.

### G.3 Camp Actions

- `camp_rest`: `NonTravel` day; HP/SAN restore; SUP cost.
- `camp_repair`: handled by C.3 as `Partial` or `NonTravel` depending on mechanic use.
- `camp_foraging`: `NonTravel` with SUP gain distribution; deterministic draw from `encounter` stream.

### G.4 Unit Tests (Phase G)

- **T‑G1**: Ticks apply multipliers exactly; integer rounding rules documented and tested.
- **T‑G2**: Camp actions produce correct deltas and `DayRecord` kind.
- **T‑G3**: Weather/EO combo bounds remain within configured min/max on fuzzed days.

---

## Phase H — **Telemetry, Metrics, and Tester**

> **Completion definition:** all tester gates derive solely from `DayRecord` + counters; warnings/errors match aggregate targets.

### H.1 Derived Metrics

- `travel_days`, `partial_travel_days`, `non_travel_days`, `travel_ratio`
- `miles_traveled`, `avg_mpd`, `days_survived`, `reached_2k_by_150`
- Crossings: `events`, `permit_uses`, `bribe_attempts`, `bribe_successes`, `detours`, `failures`
- Vehicle: `vehicle_breakdowns`, `days_with_repair`
- Narrative: `unique_encounters`, `unique_per_20_days`, `rotation_events`

### H.2 Acceptance Gates

The tester must enforce the following gates over large samples (default: 1,000 runs per scenario) using only aggregated metrics derived from `DayRecord`.

- **Global distance / duration (all policies)**

  - ERROR if `mean_miles < 1900` or `mean_miles > 2100`.
  - ERROR if `mean_days < 84` or `mean_days > 180`.
  - ERROR if `mean_mpd < 10` or `mean_mpd > 20`.
  - ERROR if `travel_ratio < 0.90` for any non-experimental policy.

- **Classic / Balanced (canonical OT profile)**

  - ERROR if `boss_reach_rate < 0.30` or `boss_reach_rate > 0.50`.
  - ERROR if `boss_win_rate < 0.20` or `boss_win_rate > 0.35`.
  - ERROR if `survival_rate < 0.60` or `survival_rate > 0.80`.
  - WARN if any single failure family (vehicle, sanity, exposure, crossings) exceeds `0.50` of all failures.

- **Other Classic strategies**

  - Must satisfy the global distance/duration/mpd and travel-ratio gates.
  - Aggressive: WARN if survival is **higher** than Classic/Balanced upper bound (the mode should be harsher, not easier).
  - Conservative / ResourceManager: WARN if boss win rate exceeds `0.40` (indicates mode has drifted into “too cozy” territory).
  - MonteCarlo: WARN if variance of miles or days is **lower** than Balanced (indicates under-exploration), while still obeying the global means.

- **Deep family (all strategies)**

  - Must satisfy the **same distance/duration/mpd and travel-ratio error bands** as Classic.
  - WARN if mean_miles or mean_days drift outside `[1900, 2100]` or `[84, 180]` even if still within loose legacy limits.
  - WARN if crossing terminal rate for Deep strategies falls below `0.08` or above `0.18` (signals crossings are either too trivial or too punishing).
  - WARN if boss reach or win rates fall outside ±50% of the Classic/Balanced targets (Deep should feel weirder, not like a completely different genre).

- **Determinism**
  - ERROR if two runs with the same seed and policy/strategy produce different CSV traces (byte-for-byte).

### H.3 Unit & Integration Tests (Phase H)

- **T‑H1**: Metrics reproduced from `DayRecord` across hand‑crafted scenarios.
- **T‑H2**: Tester aggregates reflect CSV rows from 100 simulated runs; stable to 1e‑6.
- **T‑H3**: Gate violations produce the exact WARN/ERROR strings expected by CI harness.

---

## Phase I — **Data Shapes & Migration**

> **Completion definition:** configuration lives in `journey/*.json` (or `.yaml`); legacy knobs deprecated; state serialization upgraded.

### I.1 Configuration Files

- `journey/classic.json`, `journey/deep.json` — FamilyCfg defaults.
- `journey/overlays/*.json` — Strategy overlays.
- Keep compatibility with existing vehicle & pacing files; default values drawn where available.

### I.2 Serialization & Back‑compat

- Bump `GameState` version; add `Vec<DayRecord>` and `endgame_active` flag (if missing).
- Migration fills `DayRecord` from legacy counters with a conservative reconstruction (for saved games).

### I.3 Unit Tests (Phase I)

- **T‑I1**: Config load/merge deterministic; snapshot JSON of resolved `JourneyCfg`.
- **T‑I2**: Legacy saves deserialize and upgrade; equality on non‑journey fields.
- **T‑I3**: Missing fields use defaults; error on contradictory constraints.

---

## Performance & Determinism Guarantees

- Controller tick is O(1) per day; per‑run allocations are bounded.
- No unsafe code; RNG draws constant per domain.
- CI test ensures two identical seeds yield identical SHA‑256 over exported CSV.

---

## Appendix A — Math Details & Pseudocode

### A. MPD Computation

```rust
fn compute_mpd(cfg: &JourneyCfg, state: &GameState, rng: &mut SmallRng) -> f32 {
    let mut m = cfg.mpd_base;
    m *= cfg.pace_factor[state.pace];       // steady/heated/blitz
    m *= cfg.weather_factor[state.weather];  // Clear/Storm/HeatWave/ColdSnap/Smoke
    m *= exec_multiplier(&state.exec_orders);
    m *= health_multiplier(state.hp, state.sanity);
    m = m.clamp(cfg.mpd_min, cfg.mpd_max);
    m
}
```

### B. Breakdown Check

```rust
fn check_breakdown(cfg: &JourneyCfg, wear: &mut f32, rng: &mut SmallRng, state: &GameState) -> Option<Part> {
    // Wear growth
    let fatigue = 1.0 + cfg.wear_fatigue_k * ((state.miles.max(0.0) - 1200.0).max(0.0) / 400.0);
    *wear += cfg.wear_base * cfg.pace_factor[state.pace] * cfg.weather_factor[state.weather] * fatigue;

    // Probability
    let p0 = cfg.breakdown_base; // from policy (may originate from vehicle.json anchors)
    let p  = (p0 * (1.0 + 0.5 * *wear) * cfg.pace_factor[state.pace] * cfg.weather_factor[state.weather]).min(1.0);

    if rng.gen::<f32>() < p {
        Some(sample_part(rng, &cfg.part_weights)) // weighted
    } else {
        None
    }
}
```

### C. Crossing Resolver

```rust
fn resolve_crossing(pr: &CrossingPriors, ctx: &CrossingCtx, rng: &mut SmallRng) -> CrossingOutcome {
    let mut pri = *pr;
    if ctx.has_permit { pri.p_terminal = 0.0; }
    if ctx.bribe_used  { pri = pri.apply_bribe_shift(); }

    let u = rng.gen::<f32>();
    if u < pri.p_pass {
        CrossingOutcome { kind: CrossingKind::Pass, days: 0, bribe_used: ctx.bribe_used }
    } else if u < pri.p_pass + pri.p_detour {
        let k = sample_detour_days(rng, pri.k_min, pri.k_max);
        CrossingOutcome { kind: CrossingKind::Detour(k), days: k, bribe_used: ctx.bribe_used }
    } else {
        CrossingOutcome { kind: CrossingKind::Terminal, days: 0, bribe_used: ctx.bribe_used }
    }
}
```

---

## Appendix B — Reference Defaults & UI ties

- Vehicle defaults & multipliers: base_breakdown_chance ≈ 0.005; pace_factor = { steady:1.0, heated:1.2, blitz:1.5 }; weather multipliers for Clear|Storm|HeatWave|ColdSnap|Smoke; part weights & repair costs; optional mechanic hook. These act as anchors for JourneyCfg when no explicit override is present.

- UI assets for pace/diet/weather/status are unchanged; only telemetry labels are added.

---

## Work Sequence Summary (by engineering efficiency)

1. **Phase A — Invariant Grounding**: math + `record_travel_day` + derived metrics.
2. **Phase B — Deterministic RNG**: streams, seed derivation, crossing atom, rotation discipline.
3. **Phase C — Vehicle Pipeline**: wear + breakdown + field‑repair + mechanic hook (policy‑gated).
4. **Phase D — Crossing Engine**: priors, permit/bribe, detours as partials.
5. **Phase E — Endgame Controller**: stop cap, wear suppression, finish bias, ratio guardrails.
6. **Phase F — Policy Catalog**: family defaults + strategy overlays; config loaders.
7. **Phase G — Daily Tick**: supplies/sanity/HP math; camp actions unified.
8. **Phase H — Telemetry & Tester**: aggregate metrics; acceptance gates & messages.
9. **Phase I — Data & Migration**: config files, state versioning, back‑compat tests.

---

### Unit Test Index (quick lookup)

- T‑A1..A4: accounting & records
- T‑B1..B4: determinism & RNG discipline
- T‑C1..C4: wear, breakdowns, repairs, endgame grace
- T‑D1..D4: crossings & statistics
- T‑E1..E3: endgame pacing & ratio guards
- T‑F1..F3: policy resolution
- T‑G1..G3: daily tick & camps
- T‑H1..H3: metrics & tester gates
- T‑I1..I3: config & migration
