# ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md

## Normative Systems Specification for Dystrail (Oregon Trail Deluxe ’90s Model, Modernized)

STATUS: AUTHORITATIVE / IMPLEMENTATION-BINDING
AUDIENCE: Codex, senior engineers, simulation designers
SCOPE: Core journey simulation loop, policies, events, and parity mapping

---

## 0. Intent and Sources

This document defines the authoritative simulation model for Dystrail, grounded in the
Oregon Trail Deluxe (’90s) model and mapped into Dystrail's modern systems (vehicle, exec
orders, encounters, boss gate, endgame).

Parity target (implementation-binding):
- The Oregon Trail Deluxe (DOS, version 3.0; ’90s Deluxe lineage).

Primary source (Deluxe program resources) facts used below:
- Pace help text defines Steady/Strenuous/Grueling as 8/12/16 travel hours per day and
  explicitly states strenuous/grueling increase fatigue and harm health.
- Rations help text defines Filling/Meager/Bare Bones and explicitly states Bare Bones may
  harm health.
- Occupations list includes eight professions with explicit starting cash, special advantages,
  and a final score bonus multiplier.
- Crossing UI text shows ferry cost ($5.00) and Indian guide cost (3 sets of clothes).
- Score screen strings show the item point structure (wagon x 50, oxen x 4, spare parts x 2,
  clothes x 2, bullets/50, food/25, cash/5) and that the final score is multiplied by an
  occupation bonus.

Secondary reference (used only where Deluxe resources do not expose a numeric):
- MECC Appendix-style “underlying model” summaries are used to fill in missing numeric
  thresholds (e.g., exact river-depth cutoffs, exact health scalar arithmetic). If a conflict
  is found, Deluxe overrides the older summary.

Dystrail goals:
- Preserve MECC causal ordering and physics (Weather -> Consumption -> Health -> Incidents ->
  Progress -> Crossings -> Events -> Terminal checks).
- Extend with deterministic RNG streams, policy overlays, event bus, and vehicle/boss systems.
- Keep replay determinism and phase-scoped RNG usage.

Default baseline:
- Unless explicitly stated otherwise, all Oregon Trail Deluxe thresholds and formulas in this
  document are defined under `OTDeluxe90sPolicy` (Deluxe-parity baseline).
- Dystrail-only translation layers (e.g., `general_strain` weights and `f_strain_to_prob`) MUST
  be explicitly policy-configured, because Oregon Trail Deluxe does not define Dystrail's stat axes.
- Any divergence MUST be expressed as an explicit opt-in mechanical policy overlay (e.g.,
  `DystrailLegacyPolicy`) and must not be mixed piecemeal.

---

## 1. Canonical Day Loop (MECC Model)

A Day is the atomic simulation step. Each Day must produce:
- DayInputs: player intent + derived conditions (pace, rations/diet, region, season, etc.)
- DayEffects: resource/health/progress deltas
- DayEvents[]: structured events (logged and optionally UI surfaced)
- DayRecord: audit/replay snapshot of outcomes

The Day loop is deterministic and replayable. All randomness must come from named RNG streams.

---

## 2. Terminology Mapping (OT -> Dystrail)

| MECC / Oregon Trail           | Role in Model                     | Dystrail Equivalent (normative) |
| ----------------------------- | --------------------------------- | -------------------------------- |
| Wagon                         | Mobility + inventory platform     | Vehicle (wear, breakdowns, spares) |
| Oxen                          | Speed + travel viability           | Vehicle readiness + wear proxy |
| Food (lbs)                    | Daily consumption                 | Supplies (supplies, fuel, meds) |
| Clothing                      | Cold mitigation + trade token     | Gear tags (cold/rain/smoke) + trade currency |
| Bullets                       | Hunting input + score component   | Ammo/charges resource (if hunting remains) |
| General Health (scalar)       | Collapse accumulator              | Derived general_strain + HP/Sanity/Pants outputs |
| Illness/Injury per person     | Per-member status                  | Party conditions / disease system |
| Pace                          | Speed vs risk                      | PaceId (steady/heated/blitz) |
| Rations                        | Consumption vs health             | DietId (quiet/mixed/doom) + supplies burn |
| Random events                  | Contextual hazards/opportunities  | Encounters + exec orders + weather incidents |
| Rivers                         | Hard gates                         | Crossings (checkpoint/bridge_out) |
| Forts                          | Resupply + info                    | Stops, camps, stores |
| Score                          | Post-run evaluation                | Endgame scoring |

Key translation rule: Dystrail replaces MECC's single general health scalar with multiple
visible stats (hp, sanity, pants). Parity is achieved by deriving a hidden general_strain
scalar that drives disease/event probabilities.

---

## 2.1 Political Satire Alignment (Non-Mechanical Contract)

Dystrail is political satire (Heartland -> RustBelt -> Beltway; exec orders; credibility; boss
gates). The satire must be expressed without corrupting the MECC simulation contract.

Normative rules:
- The simulation kernel MUST remain value-neutral and mechanical.
  - Events are authored as mechanical facts (`ResourceShortage`, `NavigationDelay`,
    `CorruptionAttempt`, `MutualAid`).
  - Satire lives in the presentation layer: localized copy, UI framing, character VO.
- Narrative text MUST be data-addressed (i18n keys), not embedded in core logic.
- Satirical framing MUST NOT introduce new RNG draws, hidden modifiers, or timing changes.
  - If satire needs additional flavor (extra lines, optional jokes), it MUST be deterministic
    given the same event payload.
- Satire targets systems and incentives (bureaucracy, corruption, media cycles, corporate
  influence), not protected traits or marginalized groups.

MECC events translate cleanly into satirical Dystrail wrappers while preserving mechanics:

| MECC Event Family | Mechanical Role | Dystrail Satire Wrapper (examples) |
| ----------------- | --------------- | ---------------------------------- |
| Bad/Very little water | Resource scarcity; health/progress risk | Infrastructure neglect; "boil-water advisory" |
| Inadequate grass | Mobility constraint | Supply-chain shock; "no charging stations" |
| Losing/Wrong/Lost trail | Navigation hard-stop; multi-day delay | Red-tape detour; misinfo routing; "permitting" |
| Thief coming during the night | Resource loss | Lobbyist raid; "contractor leak" |
| Indians help find food (manual phrasing) | Resource gain | Local mutual aid; community pantry |
| Passing a gravesite | Flavor + morale/strain nudge | News-cycle memorialization |
| Fire in wagon | Catastrophe; damage/loss | "PR dumpster fire" / equipment fire |

Rule of thumb: the wrapper can rename and narrate, but MUST keep the same event kind,
severity, and mechanical payload as the underlying MECC-faithful event.

---

## 3. Required State Model (MECC + Dystrail)

### 3.1 Day State (minimum MECC parity)

| Field | Type | Meaning |
| ----- | ---- | ------- |
| day | u32 | Day counter (calendar index) |
| miles_traveled | f32 | Total progress along trail |
| region | enum | Terrain band / region |
| season | enum | Derived from date |
| party | list | Party members with per-person condition |
| party_alive | u32 | Alive member count |
| health_general | i32 | General health scalar (0 best; higher is worse; death threshold is policy-defined) |
| oxen_healthy | f32 | Healthy oxen count (sick ox counts as `SICK_OX_WEIGHT`; policy-defined) |
| food_lbs | i32 | Food remaining |
| bullets | i32 | Ammo |
| clothes | i32 | Clothing |
| cash_cents | i32 | Cash in cents (USD); needed because Deluxe prices include sub-dollar units (ammo) |
| spares_wheels/axles/tongues | i32 | Spare parts |
| pace | enum | Steady / Strenuous / Grueling |
| rations | enum | Filling / Meager / Bare Bones |
| weather_today | struct | temp, precip, label |
| snow_depth | f32 | Accumulated snow |
| rain_accum | f32 | Accumulated rain |
| river_state | struct? | width/depth/swiftness when at crossing |
| wagon_state | enum | Moving / Stopped / Resting / Delayed / Blocked |
| flags/mods | map | Policy modifiers / exec orders |

### 3.2 Dystrail-specific state (normative mapping)

| Field | Type | Meaning |
| ----- | ---- | ------- |
| stats | struct | supplies, hp, sanity, morale, credibility, allies, pants, budget |
| vehicle | struct | wear, tolerance, breakdown state, spares |
| mode | enum | Classic / Deep |
| policy | enum | Balanced / Conservative / Aggressive / ResourceManager |
| encounters | struct | cooldowns, caps, recent history, diversity state |
| exec_orders | struct | active EO + multipliers + durations |
| day_state.travel | struct | blocked/delayed/partial flags |
| general_strain | f32 (derived) | Hidden scalar recomputed daily to drive odds (see 13.4) |
| boss | struct | readiness, outcome, gate status |
| endgame | struct | endgame travel scaling + boss trigger |
| weather_state | struct | today, yesterday, streaks, rain/snow accum |

---

## 4. Determinism, RNG, and Event Bus

### 4.1 RNG Streams (named domains)

All randomness must be sourced from a deterministic RngBundle with domain streams:
- rng.weather()
- rng.health()
- rng.travel()
- rng.events()
- rng.crossing()
- rng.trade()
- rng.hunt()
- rng.vehicle()/rng.breakdown()
- rng.encounter()

Rule: a phase may only consume its own stream. No cross-phase draws.

Refinement (implementation-binding):
- A phase may consume ONLY the RNG stream(s) explicitly listed for that phase in the
  normative pipeline (see 14). If a phase needs more than one stream (e.g., travel phase
  needs a navigation hard-stop roll), that must be explicitly documented and the draw
  order must be fixed.

### 4.2 Event Bus

All state changes are emitted as events (for logs and UI). Events do not drive logic.

Event {
  id, day, kind, severity, payload, tags[], ui_surface_hint
}

---

## 4.3 Mechanical Policy Overlays (OTDeluxe90s vs DystrailLegacy)

Parity requires a named mechanical policy overlay to prevent accidental drift.

Note: political satire is not a mechanical policy. It is a presentation-layer contract
(see 2.1) and MUST NOT change RNG consumption, phase order, or numerical outcomes.

### 4.3.1 OTDeluxe90sPolicy (Oregon Trail Deluxe v3.0 parity)

This overlay uses Oregon Trail Deluxe v3.0 semantics as exposed by Deluxe program resources.
Where Deluxe resources do not expose an exact numeric, the value MUST still be specified as a
policy parameter and treated as a parity-critical decision (see 16).

| Parameter | Value | Source / Notes |
| --------- | ----- | -------------- |
| PACE_MULT[steady/strenuous/grueling] | [1.0, 1.5, 2.0] | Deluxe pace help: 8/12/16 travel hours/day |
| RATIONS_ENUM | [filling, meager, bare_bones] | Deluxe rations UI/help text |
| STORE_BUY_ONLY_AT_FORTS | true | Deluxe string: “You can only buy supplies at forts.” |
| STORE_NODE_INDICES | [0, 3, 5, 8, 11, 13, 15] | OREGON.EXE store-stop name list @ `0x1EF42` (“Matt's”, “Fort Kearney”, “Fort Laramie”, “Fort Bridger”, “Fort Hall”, “Fort Boise”, “Fort Walla Walla”), mapped onto 8.5 node indices |
| CASH_UNIT | cents (USD) | Implied by Deluxe store pricing and UI (“$5.00” ferry); align with Dystrail `*_cents` fields |
| BULLETS_PER_BOX | 20 | Deluxe store help: “Each box of ammunition contains 20 bullets.” |
| STORE_BASE_PRICE_CENTS[ox] | 2000 | OREGON.EXE @ `0x1E8DA` (dword table; $20.00) |
| STORE_BASE_PRICE_CENTS[clothes_set] | 1000 | OREGON.EXE @ `0x1E8DA` (dword table; $10.00) |
| STORE_BASE_PRICE_CENTS[bullet] | 10 | OREGON.EXE @ `0x1E8DA` (dword table; $0.10 per bullet; boxes are 20) |
| STORE_BASE_PRICE_CENTS[ammo_box] | 200 | OREGON.EXE @ `0x1EE24` (dword table; $2.00 per box; equals `10 * BULLETS_PER_BOX`) |
| STORE_BASE_PRICE_CENTS[food_lb] | 20 | OREGON.EXE @ `0x1E8DA` (dword table; $0.20 per pound) |
| STORE_BASE_PRICE_CENTS[wheel/axle/tongue] | 1000 | OREGON.EXE @ `0x1E8DA` (dword table; $10.00 each) |
| STORE_PRICE_MULT_PCT_BY_NODE | [100,100,100,100,125,125,150,150,150,175,175,175,200,200,225,250,250,250,250] | OREGON.EXE @ `0x1E8DA + 0x1C`; normative mapping: first 18 entries align to 8.5 node indices (0..17); trailing entry is an extra post-arrival stage (still 250) |
| STORE_MAX_BUY[oxen] | 20 | OREGON.EXE @ `0x1EE40` (u16 list; store input constraint) |
| STORE_MAX_BUY[ammo_boxes] | 50 | OREGON.EXE @ `0x1EE40` (u16 list; store input constraint) |
| STORE_MAX_BUY[clothes_sets] | 99 | OREGON.EXE @ `0x1EE40` (u16 list; store input constraint) |
| STORE_MAX_BUY[wheels/axles/tongues] | 3 | OREGON.EXE @ `0x1EE40` (u16 list; store input constraint) |
| STORE_MAX_BUY[food_lbs] | 2000 | OREGON.EXE @ `0x1EE40` (u16 list; store input constraint) |
| FERRY_COST_CENTS | 500 | Deluxe crossing UI: “Take ferry for $5.00” |
| GUIDE_COST_CLOTHES | 3 | Deluxe crossing UI: “3 sets of clothes” |
| OCCUPATIONS | [banker, doctor, merchant, blacksmith, carpenter, saddlemaker, farmer, teacher] | Deluxe occupation help |
| OCC_STARTING_CASH_DOLLARS | [1600, 1200, 1200, 800, 800, 800, 400, 400] | Deluxe occupation help; applied as `cash_cents = dollars * 100` |
| OCC_FINAL_BONUS_MULT | [1.0, 1.0, 1.5, 2.0, 2.0, 2.5, 3.0, 3.5] | Deluxe occupation help (“none” => x1.0) |
| OCC_ADV_DOCTOR | enabled | Deluxe: “sick or injured people are less likely to die” (numeric effect policy-defined) |
| OCC_ADV_BLACKSMITH | enabled | Deluxe: “more likely to repair broken wagon parts” (numeric effect policy-defined) |
| OCC_ADV_CARPENTER | enabled | Deluxe: “more likely to repair broken wagon parts” (numeric effect policy-defined) |
| OCC_ADV_FARMER | enabled | Deluxe: “oxen are less likely to get sick and die” (numeric effect policy-defined) |
| OCC_DOCTOR_FATALITY_MULT | 0.50 (default) | Q11: multiplies any “death while sick/injured” checks (disease fatality, complications); tune later via Monte Carlo/EXE |
| OCC_REPAIR_SUCCESS_MULT | 1.25 (default) | Q11: applies to Blacksmith/Carpenter; multiplies repair success probabilities (or divides failure odds) |
| OCC_MOBILITY_FAILURE_MULT | 0.75 (default) | Q11: applies to Farmer; multiplies ox sickness/death (or Dystrail mobility-failure analogue) odds |
| HEALTH_MAX | unbounded | MECC Appendix B model: scalar is 0..140+ (no hard max); death threshold at 140 |
| HEALTH_DEATH_THRESHOLD | 140 | MECC Appendix B: “remaining party members all die within a few days” at ≥140; Deluxe UI label set is consistent |
| HEALTH_LABEL_RANGES | [0-34 good, 35-69 fair, 70-104 poor, 105-139 very_poor, ≥140 death_imminent] | MECC Appendix B; treat as Deluxe-parity baseline unless contradicted by EXE/empirical |
| HEALTH_RECOVERY_BASELINE | `health_general -= 10` | MECC Appendix B: “general health value … is decremented by 10, representing natural recovery.” |
| P_AFFLICTION_MAX | 0.40 | MECC/Deluxe model family: 0%..40% (not explicitly exposed in Deluxe UI text) |
| AFFLICTION_CURVE_PWL | [(0,0.00)->(34,0.05); (35,0.05)->(69,0.15); (70,0.15)->(104,0.25); (105,0.25)->(139,0.40); (>=140,0.40)] | OTDeluxe90sPolicy default (Q2): piecewise-linear monotone curve; empirical-fit later without changing kernel |
| ILLNESS_DURATION_DAYS | 10 | MECC Appendix B: illness recovery is 10 days (Deluxe may still present named illnesses) |
| INJURY_DURATION_DAYS | 30 | MECC Appendix B: injury recovery is 30 days |
| DISEASE_CATALOG | policy-defined | Q12: data-driven catalog of named diseases/injuries (weights/effects/fatality); seeded from Deluxe strings; refined via sampling |
| AFFLICTION_REPEAT_KILLS | true | MECC Appendix B: if a member already sick/injured is selected again, they die |
| BASE_MPD_PLAINS_STEADY_GOOD | 20 | MECC Appendix B: steady good plains with ≥4 healthy oxen => 20 miles/day |
| TERRAIN_MULT[mountains] | 0.5 | MECC Appendix B: mountains halve travel speed |
| OXEN_MIN_FOR_BASE | 4 | MECC Appendix B: base speed assumes ≥4 healthy oxen |
| OXEN_MIN_TO_MOVE | 1.0 effective oxen | Deluxe strings only hard-block when effective oxen is 0 (“no oxen” / “only ox is sick”) |
| SICK_OX_WEIGHT | 0.5 | MECC Appendix B: sick ox counts as ½ a healthy ox |
| SICK_MEMBER_SPEED_PENALTY | 0.10 per sick member | MECC Appendix B: each sick party member reduces speed by 10% |
| TRAIL_MILEMARKERS_MAIN | see 8.5 | OREGON.EXE @ `0x1D3AA` (u16 mile markers; main route) |
| TRAIL_MILEMARKERS_SUBLETTE | see 8.5 | OREGON.EXE @ `0x1D3D0` (u16 mile markers; includes 0 sentinel at Fort Bridger index) |
| TRAIL_MILEMARKERS_DALLES_SHORTCUT | see 8.5 | OREGON.EXE @ `0x1D3F6` (u16 mile markers; includes 0 sentinel at Fort Walla Walla index) |
| TRAIL_MILEMARKERS_SUBLETTE_AND_DALLES_SHORTCUT | see 8.5 | OREGON.EXE @ `0x1D41C` (u16 mile markers; includes 0 sentinels at Fort Bridger and Fort Walla Walla indices) |
| TRAIL_TOTAL_MILES_MAIN | 2083 | Derived from `TRAIL_MILEMARKERS_MAIN` endpoint (Willamette Valley) |
| SUBLETTE_CUTOFF_DISTANCE_MILES | 125 | Derived: South Pass (932) -> Green River (1057) on Sublette route |
| SUBLETTE_CUTOFF_SAVES_MILES | 94 | Derived: main Green River (1151) vs Sublette Green River (1057) and same delta thereafter |
| DALLES_SHORTCUT_DISTANCE_MILES | 125 | Derived: Blue Mountains (1808) -> The Dalles (1933) on shortcut route |
| DALLES_SHORTCUT_SAVES_MILES | 50 | Derived: main The Dalles (1983) vs shortcut The Dalles (1933) and same delta thereafter |
| FERRY_WAIT_DAYS | 0..6 (inclusive) | MECC Appendix B: must wait up to 6 days |
| FERRY_WAIT_DAYS_DISTRIBUTION | uniform(0..6) (default) | OTDeluxe90sPolicy default (Q4); log sampled wait days to enable later empirical fit |
| FERRY_RISK_RANGE | 0.0..0.10 (by swiftness; function policy-defined) | MECC Appendix B: accident risk varies 0%..10% based on river swiftness |
| FERRY_ACCIDENT_OUTCOME_WEIGHTS | policy-defined (default excludes death) | OTDeluxe90sPolicy default (Q5): accident bucket causes wet goods/time loss/minor loss; no ferry deaths unless confirmed |
| FERRY_MIN_DEPTH_FT | 2.5 | MECC Appendix B: no ferry when river drops below 2.5 ft |
| CAULK_FLOAT_MIN_DEPTH_FT | 1.5 | MECC Appendix B: river must be at least 1.5 ft deep to caulk/float |
| CAULK_FLOAT_HELP_RECOMMENDED_MIN_DEPTH_FT | 2.5 | Deluxe help: only attempt caulking/float in water > 2.5 ft (guidance; not a mechanical minimum) |
| FORD_RECOMMENDED_MAX_DEPTH_FT | 2.5 | Deluxe help guidance; also aligns with MECC wet-goods threshold (2.5..3.0) |
| FORD_WET_GOODS_MIN_DEPTH_FT | 2.5 | MECC Appendix B: between 2.5 and 3.0 ft, goods get wet and time is lost drying out |
| FORD_SWAMP_DEPTH_FT | 3.0 | MECC Appendix B: wagon swamps past 3.0 ft; losses scale with depth |
| FORD_SWIFTNESS_RISK_CURVE | policy-defined | OTDeluxe90sPolicy decision (Q7): swiftness is continuous; do not invent a binary “slow-moving” threshold |
| CROSSING_OUTCOME_WEIGHTS | policy-defined | Not surfaced; must be specified (safe/stuck_in_mud/supplies_wet/tipped/sank/drown) |
| WET_SUPPLIES_EFFECT | drying_days=1, permanent_loss=none (default) | OTDeluxe90sPolicy default (Q9): wet goods cost a day drying; no permanent loss unless swamped; Dystrail translation may apply minor stress/supplies deltas |
| GUIDE_RISK_MULT | 0.20 | MECC Appendix B: guide reduces accident risk by 80% |
| GUIDE_LOSS_MULT | 0.50 (default) | OTDeluxe90sPolicy default (Q6): guide also reduces loss magnitude on accidents (qualitative in MECC; tune later) |
| CROSSING_COST_DAYS | 1 | MECC Appendix B: caulk/float and ford each consume 1 full day away from the trail (adopted for Deluxe parity unless contradicted by Deluxe evidence) |
| DRYING_COST_DAYS | 1 | MECC Appendix B: wet goods between 2.5..3.0 costs a day drying out |
| REST_DAYS_RANGE | 1..9 | Deluxe rest UI: “You may rest for 1 to 9 days.” |
| TRADE_COST_DAYS | 1 | MECC Appendix B: each trade costs a day away from the trail (adopted for Deluxe parity unless contradicted by Deluxe evidence) |
| HUNT_COST_DAYS | 1 | MECC Appendix B: hunting takes a day away from the trail (adopted for Deluxe parity unless contradicted by Deluxe evidence) |
| HUNT_CARRY_CAP_LBS | `100 * alive_party_members` | MECC Appendix B: 100 lbs per carrier; Deluxe screenshots strongly support scaling by survivors; OTDeluxe90sPolicy decision (Q3): injuries do not reduce carry cap |
| WAGON_CAPACITY_MODEL | per_item_caps | OTDeluxe90sPolicy decision (Q10): assume per-item caps only unless a distinct total-weight model is proven |
| WAGON_CAPACITY_TOTAL_LBS | policy-defined (optional) | Only if Deluxe evidence shows a total-weight model exists; requires extraction of total limit and per-item weights |
| BARLOW_TOLL_ROAD_COST_CENTS | policy-defined | Deluxe string: “You cannot afford to take the Barlow Toll Road.” (cost not yet extracted) |
| BARLOW_TOLL_ROAD_TIME_DAYS | policy-defined | Deluxe time cost not yet extracted |
| RAFTING_TIME_DAYS | policy-defined | Deluxe rafting time cost not yet extracted (may be 0 if resolved as an immediate outcome; must be explicit) |
| RAFTING_OUTCOME_WEIGHTS | policy-defined | Deluxe rafting flow exists (`RAFTING.*` resources), but outcome weights/time cost not yet extracted |
| SCORE_ITEM_POINTS[wagon/ox/spare_part/clothes] | [50, 4, 2, 2] | Deluxe score screen strings |
| SCORE_ITEM_DIVISORS[bullets/food/cash_cents] | [50, 25, 500] | Deluxe score screen strings; cash divisor is $5.00 => 500 cents |
| SCORE_POINTS_PER_PERSON_BY_HEALTH | {good:500,fair:0,poor:0,very_poor:0} (default) | Deluxe capture confirms Good=500; OTDeluxe90sPolicy default (Q16): non-Good tiers are 0 until proven otherwise |
| DEATH_IMMINENT_GRACE_DAYS | policy-defined (default 3) | “all die within a few days” semantics (see 6.5) |
| DEATH_IMMINENT_RESET_MODE | reset_on_recovery_below_threshold (default) | OTDeluxe90sPolicy default (Q17): if health recovers below threshold, death timer resets |
| STRAIN_NORM_DENOM | policy-defined | Q18: normalizes `general_strain` into 0..1 for label/scoring translation |
| STRAIN_LABEL_BOUNDS | [0.25, 0.50, 0.75] (default) | Q18: `general_strain_norm` cutoffs for Good/Fair/Poor/VeryPoor |

Unspecified-by-Deluxe-resource mappings MUST be expressed as policy parameters (not hard-coded), with
defaults chosen to be monotonic and bounded:
- affliction curve shape is policy-defined (default `AFFLICTION_CURVE_PWL`) and clamped to `P_AFFLICTION_MAX`
- M_snow(snow_depth) in [0..1]
- delay_days_remaining for navigation hard-stops
- caulk/ford accident risk functions (beyond the listed thresholds)
- precip evaporation and snow-melt rates (manual describes the presence of both, not magnitudes)

### 4.3.2 DystrailLegacyPolicy (optional: preserve Dystrail mechanics)

This overlay exists to preserve (or emulate) currently observed Dystrail behaviors that are
not explicitly specified by Oregon Trail Deluxe resources. It may diverge from OTDeluxe90sPolicy, but MUST
remain deterministic and fully specified.

It is not a parity target; it is an explicit opt-in for legacy behavior.

Examples of legacy divergences (see 16 for the explicit checklist):
- hunting carry cap scaling by survivors
- using Dystrail-only currencies/resources instead of Deluxe itemized inventory

---

## 5. Weather System (Deluxe Causality, Dystrail Generator)

### 5.1 Generation sources and parity stance

MECC Appendix B describes a Deluxe-lineage weather generator driven by monthly temperature and rainfall tables
for six climate stations near the historic trail (Kansas City, North Platte, Casper, Lander, Boise, Portland).

Dystrail parity decision (Q13):
- Do not attempt to clone those six-station tables for parity v1.
- Keep Dystrail’s existing `weather.json` regional weighting model as the active generator.
- Preserve Deluxe/MECC causal precedence and fan-out: weather is the root cause that feeds health, travel, crossings,
  and event probabilities.

Normative abstraction:

WeatherModel {
  generate_weather_today(context, rng.weather()) -> WeatherToday
  compute_weather_effects(context, WeatherToday) -> WeatherEffects
}

Required implementations:
- `DystrailRegionalWeather` (default): uses `weather.json` weights by Dystrail region/season.
- `OTDeluxeStationsWeather` (optional future): uses climate-station tables and the Deluxe sampling procedure.

If `OTDeluxeStationsWeather` is later selected, policy must provide:
- `TEMP_MEAN[station][month]` (°F) and `RAIN_MEAN[station][month]` (in/month liquid equivalent)
- the daily sampling distribution (variance/shape)
- precip thresholds for “rainy/very rainy/snowy/very snowy”

### 5.2 Weather Report Labels (MECC)

If precipitation is present:
- label = rainy / snowy / very rainy / very snowy

Otherwise, label is based on temperature band:
- very hot: > 90 F
- hot: 70-90 F
- warm: 50-70 F
- cool: 30-50 F
- cold: 10-30 F
- very cold: < 10 F

### 5.3 Accumulation and Evaporation

- Snowfall accumulates on the ground as snow_depth.
- Rainfall accumulates as surface water (rain_accum).
- Each day, some accumulated rain/snow evaporates.
- On warm days, some snow melts and becomes water.

Parity stance (Q14):
- Even if the active generator (`DystrailRegionalWeather`) never emits snow in early parity builds, the kernel/state
  MUST carry `snow_depth` and the slowdown hook (`M_snow(snow_depth)`) so Deluxe winter behavior can be added later
  without refactoring.

### 5.4 Weather Fan-out (normative)

Weather must be the root cause in the daily pipeline.

Weather -> { HealthDelta, RiverDepthDelta, ProgressDelta, EventProbDelta }

Dystrail may keep its weather.json weighting model, but MUST preserve this causal fan-out.

Required WeatherEffects struct:

WeatherEffects {
  travel_speed_factor,
  supplies_drain_delta,
  sanity_delta,
  pants_delta,
  encounter_chance_delta,
  breakdown_chance_delta,
  crossing_risk_delta,
  river_depth_delta,
}

---

## 6. Health System (Oregon Trail Deluxe lineage)

### 6.1 General Health Scalar

`health_general` is an integer scalar where:
- 0 is ideal
- higher values are worse
- the death-imminent threshold is `HEALTH_DEATH_THRESHOLD = 140` (MECC Appendix B; see 4.3.1 and 16)

Display mapping ranges (MECC Appendix B; adopted for Deluxe parity):
- 0-34: Good
- 35-69: Fair
- 70-104: Poor
- 105-139: Very Poor
- >= 140: Death imminent (party wipe within days)

Implementation-binding (Q1):
- These thresholds MUST live in policy/config (e.g., `OTDeluxe90sPolicy.health.thresholds`) and must not be hard-coded in the kernel.

### 6.2 Daily Update (series model)

MECC manual describes a natural recovery step followed by additive factors:
- weather (temperature, precipitation)
- pace (or resting)
- clothing (during winter)
- individual illnesses or injuries
- food rations (or lack of food)
- random events (especially drought)

Normative formula (MECC-faithful):

health_general_next = max(0,
  (health_general - 10)                    // MECC Appendix B: natural recovery baseline
  + H_weather(weather_today)
  + H_pace(pace)
  + H_rations(rations)
  + H_clothing(season, clothes)
  + H_afflictions(party)
  + H_drought(rain_accum, season)
  + sum(H_event(e) for e in events_today)   // optional; only for events with health payloads
)

Note:
- The equation above describes end-of-day health. In an event-sourced kernel, the
  `sum(H_event(e))` term may be applied during event resolution later in the day, so long as
  the net effect matches the equation and no event is double-counted.

### 6.3 Illness/Injury Incidence

On any day, odds of a party member becoming sick or injured ranges 0%..40%
depending on general health.

Normative (implementation-friendly; policy-configurable):

p_affliction_today =
  clamp(
    affliction_curve_pwl(health_general, policy),
    0.0,
    P_AFFLICTION_MAX
  )

OTDeluxe90sPolicy default curve (Q2):
- Piecewise-linear within the MECC health label ranges:
  - health 0..34:   0.00 -> 0.05
  - health 35..69:  0.05 -> 0.15
  - health 70..104: 0.15 -> 0.25
  - health 105..139: 0.25 -> 0.40
  - health >= 140:  0.40 (clamped)

If triggered:
- choose person and affliction randomly
- if chosen person already sick/injured -> person dies
- duration/recovery (MECC Appendix B; adopted for Deluxe parity):
  - illness recovery: `ILLNESS_DURATION_DAYS = 10`
  - injury recovery: `INJURY_DURATION_DAYS = 30`
  - if selected again while already sick/injured: death (`AFFLICTION_REPEAT_KILLS = true`)
- Deluxe flavor note (supports Q11):
  - Deluxe includes explicit disease-death strings (e.g., “died of cholera/typhoid/measles/dysentery”), implying that
    active illnesses may include fatality checks beyond “health worsens”.
  - OTDeluxe90sPolicy therefore defines per-disease fatality mechanics as part of a data-driven disease catalog, and
    Doctor perks apply via `OCC_DOCTOR_FATALITY_MULT` (see 4.3.1).

Deluxe presentation note:
- Deluxe surfaces named conditions (cholera, measles, dysentery, typhoid, exhaustion, fever, broken arm/leg).
  For parity, implement a named catalog for UI/log flavor, but do not assume per-disease durations unless Deluxe
  evidence contradicts the MECC 10/30-day model.

### 6.3.1 Disease Catalog (OTDeluxe90sPolicy; data-driven)

OTDeluxe90sPolicy requires a named disease/affliction catalog for parity with Deluxe presentation and downstream
mechanics. This is intentionally data-driven so we can re-fit Deluxe incidence/effects later without touching the
kernel.

Normative schema (implementation-binding):

DiseaseDef {
  id,                     // stable mechanical identifier (not localized)
  kind,                   // illness | injury
  display_key,            // i18n key for UI/logs (satire-safe wrapper applied at presentation)
  duration_days,          // default: 10 for illness, 30 for injury (MECC Appendix B), unless Deluxe evidence proves otherwise
  onset_effects,          // immediate deltas (e.g., health penalties, supplies, travel slowdown)
  daily_tick_effects,     // per-day deltas while active
  fatality_model,         // optional; if present, defines death-check odds while active (Deluxe strings imply some diseases can be fatal)
  tags[],                 // e.g., "waterborne", "injury", "snakebite"
}

Fatality model (policy-driven; supports Doctor perk):

FatalityModel {
  base_prob_per_day,      // policy-defined; default 0 unless evidence shows lethality
  prob_modifiers[],       // functions of context (health label, pace, rations, weather)
  apply_doctor_mult,      // if true, multiply final probability by `OCC_DOCTOR_FATALITY_MULT`
}

Determinism rules:
- All disease selection and disease-fatality rolls use `rng.health()`.
- The catalog is the single source of truth for incidence/effects; UI copy must never change mechanics.

### 6.4 Travel Speed Effect (series model)

Each sick party member reduces speed by 10% (MECC Appendix B; adopted for Deluxe parity).

Normative travel multiplier:

M_party_sick(sick_count) = max(0.0, 1.0 - (0.10 * sick_count))

### 6.5 Death-Imminent Threshold Semantics (series model)

Manual statement (series model): when `health_general >= HEALTH_DEATH_THRESHOLD`, “remaining party members all die within a few days.”

Implementation-binding rule (because “a few days” is underspecified):

- On any Day where `health_general >= HEALTH_DEATH_THRESHOLD`, enforce:
  `death_imminent_days_remaining = min(death_imminent_days_remaining, DEATH_IMMINENT_GRACE_DAYS)`
  (so the timer never increases while death is imminent).
- At end of each subsequent Day where `health_general >= HEALTH_DEATH_THRESHOLD`, decrement
  `death_imminent_days_remaining` by 1.
- When `death_imminent_days_remaining` reaches 0, the party dies (terminal failure).
- If `health_general` drops back below `HEALTH_DEATH_THRESHOLD` before the countdown reaches 0,
  reset/clear `death_imminent_days_remaining` (policy may choose whether the reset is immediate or
  requires N consecutive “non-imminent” days; default is immediate reset).

---

## 7. Supplies and Rations (MECC -> Dystrail)

MECC defines rations qualitatively:
- filling: meals are large and generous
- meager: meals are small but adequate
- bare bones: meals are very small; everyone stays hungry

Rations affect both food consumption and health penalties.

Dystrail mapping: DietId (quiet/mixed/doom) controls supplies burn and health penalties.

Normative Dystrail supplies burn formula:

supplies_burn =
  base_supplies_burn(region)
  * pace_supplies_factor(pace)
  * weather_supplies_factor(weather)
  * vehicle_supplies_factor(vehicle_state)
  * exec_supplies_factor(exec_orders)
  * diet_supplies_factor(diet)

stats.supplies -= supplies_burn

If supplies <= 0, starvation mechanics apply (existing Dystrail constants).

---

## 8. Travel and Progress (Oregon Trail Deluxe lineage)

### 8.0 Travel viability gates (Deluxe strings)

Deluxe includes hard-block conditions where the wagon cannot continue moving until corrected:
- No oxen: travel is blocked (“You have no oxen. You must get another ox to continue.”).
- Only ox is sick: travel is blocked (“Your only ox is sick. You must get another ox to continue.”).
- Broken wagon parts can hard-block travel until repaired or replaced (e.g., wagon tongue help: “If the wagon tongue
  breaks, you have to repair or replace it before you can continue on the trail.”).

Normative requirement:
- Travel is allowed iff `effective_oxen >= OXEN_MIN_TO_MOVE` (MECC/Deluxe model family; Deluxe strings explicitly
  hard-block at 0 effective oxen).
- For Deluxe parity, use `OXEN_MIN_TO_MOVE = 1.0` effective oxen (see 4.3.1).

### 8.1 Base miles/day

If:
- plains terrain
- steady pace
- at least `OXEN_MIN_FOR_BASE = 4` effective oxen
- health is not in a collapsed state (not death-imminent)
Then the base travel rate is `BASE_MPD_PLAINS_STEADY_GOOD = 20` miles/day (MECC Appendix B; adopted for Deluxe parity).

Notes:
- Deluxe pace help exposes travel time (8/12/16 hours/day). MECC Appendix B provides the underlying
  miles/day arithmetic; we adopt it as the Deluxe parity baseline unless contradicted by EXE/empirical evidence.

### 8.2 Multipliers (series model)

Pace:
- Steady: x1.0
- Strenuous: x1.5
- Grueling: x2.0

Terrain:
- Mountains: `TERRAIN_MULT[mountains] = 0.5` (MECC Appendix B)

Oxen:
- if `< OXEN_MIN_FOR_BASE`: multiply by `(effective_oxen / OXEN_MIN_FOR_BASE)`
- sick ox contribution: `SICK_OX_WEIGHT = 0.5` (MECC Appendix B)

Sick party:
- each sick party member reduces speed by 10% (MECC Appendix B; see 6.4 for multiplier)

Snow:
- up to 100% loss depending on snow depth (`M_snow(snow_depth)` must be monotone in [0..1]; policy-defined)

### 8.3 Normative Progress Formula

miles_today = BASE_MPD_PLAINS_STEADY_GOOD
  * M_pace(pace)
  * M_terrain(region)
  * M_oxen(oxen_healthy)
  * M_party_sick(party)
  * M_snow(snow_depth)
  + M_random_adjust(events_today)

### 8.4 Hard-stops

Events can completely halt progress for several days:
- impassable trail
- lost trail
- wrong trail

If any hard-stop event occurs:

miles_today = 0
wagon_state = Delayed or Blocked
delay_days_remaining += D(event)

### 8.5 Trail Graph and Landmark Mile Markers (Deluxe EXE-extracted)

Deluxe progression is defined over a fixed ordered list of trail nodes. Each node has a
"mile marker" (cumulative miles from the start) used for:
- triggering landmark arrival
- determining which store (if any) is available
- determining the current pricing multiplier (see 4.3.1)
- determining the end-of-trail arrival threshold

Source of truth:
- OREGON.EXE @ `0x1D440`: node name list (18 strings; includes start + Willamette Valley)
- OREGON.EXE @ `0x1D3AA`: u16 mile-marker list for the main route (17 values; excludes the start at mile 0)
- OREGON.EXE @ `0x1D3D0`: u16 mile-marker list for the Sublette Cutoff route variant (contains a 0 sentinel where
  Fort Bridger is skipped)
- OREGON.EXE @ `0x1D3F6`: u16 mile-marker list for the "shortcut to The Dalles" route variant (contains a 0
  sentinel where Fort Walla Walla is skipped)
- OREGON.EXE @ `0x1D41C`: u16 mile-marker list for taking both Sublette Cutoff and the Dalles shortcut (contains
  0 sentinels at Fort Bridger and Fort Walla Walla indices)

#### 8.5.1 Trail nodes (main route order)

| Node Index | Location (Deluxe string) | Mile Marker (main) | Segment Miles (from previous) |
| ---------- | ------------------------- | ------------------ | ----------------------------- |
| 0 | Independence, Missouri | 0 | - |
| 1 | the Kansas River Crossing | 102 | 102 |
| 2 | the Big Blue River Crossing | 185 | 83 |
| 3 | Fort Kearney | 304 | 119 |
| 4 | Chimney Rock | 554 | 250 |
| 5 | Fort Laramie | 640 | 86 |
| 6 | Independence Rock | 830 | 190 |
| 7 | South Pass | 932 | 102 |
| 8 | Fort Bridger | 989 | 57 |
| 9 | the Green River Crossing | 1151 | 162 |
| 10 | Soda Springs | 1295 | 144 |
| 11 | Fort Hall | 1352 | 57 |
| 12 | the Snake River Crossing | 1534 | 182 |
| 13 | Fort Boise | 1648 | 114 |
| 14 | Grande Ronde in the Blue Mountains | 1808 | 160 |
| 15 | Fort Walla Walla | 1863 | 55 |
| 16 | The Dalles | 1983 | 120 |
| 17 | the Willamette Valley | 2083 | 100 |

Implementation-binding note:
- The main-route end-of-trail threshold for arrival is `TRAIL_TOTAL_MILES_MAIN = 2083` (Willamette Valley).
- Deluxe contains additional map/label strings for “Oregon City”; for parity, treat “Oregon City” as an in-valley
  presentation label only (no additional simulation node or miles beyond Willamette Valley).

#### 8.5.2 Sublette Cutoff (Deluxe distance table evidence)

Deluxe encodes an alternate mile-marker list that skips Fort Bridger and shortens the trail
after South Pass. The data table indicates:
- The branch point is South Pass (`932`).
- The Sublette Cutoff path goes directly from South Pass to the Green River Crossing:
  `SUBLETTE_CUTOFF_DISTANCE_MILES = 1057 - 932 = 125`.
- Fort Bridger is skipped (0 sentinel in the EXE table at the Fort Bridger index).
- All later mile markers shift by `SUBLETTE_CUTOFF_SAVES_MILES = 94` (e.g., Green River `1151 -> 1057`).
- The end-of-trail threshold becomes `2083 - 94 = 1989` (Willamette Valley).

Parity contract:
- When Sublette Cutoff is chosen, the simulation MUST remove/skip the Fort Bridger node for:
  - store availability at that node (no fort stop)
  - the store-price multiplier schedule and any location-based scaling that uses node index
  - arrival triggers keyed to Fort Bridger

#### 8.5.3 Shortcut to The Dalles (Deluxe distance table evidence)

Deluxe encodes an alternate mile-marker list that skips Fort Walla Walla and shortens the
trail after the Blue Mountains. The data table indicates:
- The branch point is the Blue Mountains node (`1808`).
- The shortcut path goes directly from Blue Mountains to The Dalles:
  `DALLES_SHORTCUT_DISTANCE_MILES = 1933 - 1808 = 125`.
- Fort Walla Walla is skipped (0 sentinel in the EXE table at the Fort Walla Walla index).
- All later mile markers shift by `DALLES_SHORTCUT_SAVES_MILES = 50` (e.g., The Dalles `1983 -> 1933`).
- The end-of-trail threshold becomes `2083 - 50 = 2033` (Willamette Valley).

Parity contract:
- When the shortcut is chosen, the simulation MUST remove/skip the Fort Walla Walla node for:
  - store availability at that node (no fort stop)
  - arrival triggers keyed to Fort Walla Walla

#### 8.5.4 Sublette Cutoff + Dalles shortcut (combined table; Deluxe evidence)

Deluxe encodes a fourth mile-marker list for the case where the player takes BOTH:
- Sublette Cutoff at South Pass (skips Fort Bridger), AND
- the shortcut to The Dalles after the Blue Mountains (skips Fort Walla Walla).

EXE evidence (OREGON.EXE @ `0x1D41C`) indicates:
- Fort Bridger is skipped (0 sentinel at the Fort Bridger index).
- Fort Walla Walla is skipped (0 sentinel at the Fort Walla Walla index).
- All later mile markers shift by the combined savings:
  `SUBLETTE_AND_DALLES_SAVES_MILES = SUBLETTE_CUTOFF_SAVES_MILES + DALLES_SHORTCUT_SAVES_MILES = 144`.
- The Dalles mile marker becomes `1983 - 144 = 1839`.
- The end-of-trail threshold becomes `2083 - 144 = 1939` (Willamette Valley).

Parity contract:
- If both choices are taken, the simulation MUST skip both Fort Bridger and Fort Walla Walla for:
  - store availability at those nodes
  - arrival triggers keyed to those nodes
  - any location-indexed schedules keyed to node index (including store multipliers)

---

## 9. River Crossing System (Oregon Trail Deluxe lineage)

### 9.1 Rivers modeled

- Kansas
- Big Blue
- Green
- Snake
- Columbia (special endgame rafting choice at The Dalles; see 9.5)

### 9.2 River State Inputs

Depth/width/swiftness derived from:
- minimum values per river
- recent rainfall accumulation
- seasonal highs (March/April) and summer decline

### 9.3 Options and Constraints (series model)

| Option | Constraints | Cost/Time | Risk Notes |
| ------ | ---------- | --------- | ---------- |
| Ferry | Only where historically exists | `$FERRY_COST_CENTS` + `sample_ferry_wait_days()` | Lowest risk; risk range is `FERRY_RISK_RANGE`; availability depends on `FERRY_MIN_DEPTH_FT` |
| Caulk & float | Requires: `depth >= CAULK_FLOAT_MIN_DEPTH_FT` | `CROSSING_COST_DAYS` (default 1) | Risk varies with depth+swiftness; may include sink/capsize and drownings |
| Ford | Always selectable (risk cliffs by depth/swiftness) | `CROSSING_COST_DAYS` (default 1) | Low risk when shallow/slow; wet goods in 2.5..3.0; swamps past 3.0; muddy/rocky banks add stuck/overturn risk |
| Hire guide | Snake only | `GUIDE_COST_CLOTHES` | Accident risk multiplier `GUIDE_RISK_MULT` (MECC: 0.20) and loss reduction on accidents (MECC; magnitude policy-defined) |

### 9.4 Crossing Thresholds, Risk Cliffs, and Required Outcomes (MECC + Deluxe evidence)

MECC Appendix B describes the underlying river-crossing mechanics. Deluxe UI/help text is consistent with the same
model, but does not always print the numeric thresholds.

#### 9.4.1 Ferry mechanics (MECC explicit)

- Availability: no ferry when `depth < FERRY_MIN_DEPTH_FT = 2.5`.
- Cost: `$5.00` (`FERRY_COST_CENTS = 500`).
- Wait time: must wait up to 6 days (`FERRY_WAIT_DAYS = 0..6`, inclusive).
  - OTDeluxe90sPolicy default (Q4): uniform distribution over `0..6` (policy-configurable).
- Accident risk: varies from 0% to 10% based on river swiftness (`FERRY_RISK_RANGE = 0.0..0.10`; function unspecified).
  - OTDeluxe90sPolicy default (Q5): ferry accidents are an “incident bucket” with non-lethal outcomes only (wet goods,
    time loss, minor loss); do not allow party death on ferry until confirmed by Deluxe evidence.

#### 9.4.2 Caulk & float mechanics (MECC explicit; Deluxe help guidance)

- Mechanical minimum: river must be at least `CAULK_FLOAT_MIN_DEPTH_FT = 1.5` deep.
- Risk varies from low to very high based on depth and swiftness (functional form and weights are not specified in
  the manual and must be locked by EXE analysis or empirical sampling).
- Deluxe help guidance (not a mechanical minimum): only attempt caulking/float in water more than
  `CAULK_FLOAT_HELP_RECOMMENDED_MIN_DEPTH_FT = 2.5`.

#### 9.4.3 Fording mechanics (MECC explicit risk cliffs; Deluxe outcomes confirm)

MECC Appendix B defines two depth cliffs with specific effects:
- If `depth < FORD_WET_GOODS_MIN_DEPTH_FT = 2.5`: low risk (assuming non-extreme swiftness/banks).
- If `2.5 <= depth <= 3.0`: goods get wet and 1 day is wasted drying out (`DRYING_COST_DAYS = 1`).
  - OTDeluxe90sPolicy default (Q9): wet goods implies time loss but no permanent inventory loss unless the wagon swamps.
- If `depth > FORD_SWAMP_DEPTH_FT = 3.0`: wagon swamps; losses scale with depth.

Additional fording hazards:
- muddy banks increase chance of getting stuck
- rocky banks increase chance of overturning
- swiftness is a continuous risk input (Q7): do not invent a binary “slow-moving water” threshold; risk increases with swiftness.

Deluxe in-game strings confirm outcomes consistent with this model: supplies wet, wagon tipped, wagon sank.

#### 9.4.4 Hire guide mechanics (MECC explicit; Deluxe UI confirms cost)

- Only available at Snake River.
- Cost: exactly 3 sets of clothing (`GUIDE_COST_CLOTHES = 3`).
- Accident risk reduction: 80% (`GUIDE_RISK_MULT = 0.20`).
- Loss reduction on accidents: MECC states losses are reduced, but does not provide a numeric multiplier.
  OTDeluxe90sPolicy default (Q6): apply a multiplicative loss reduction (`GUIDE_LOSS_MULT = 0.50`) and tune later via sampling.

Required crossing outcome families (surfaced by Deluxe in-game strings) include:
- Safe crossing
- Stuck in muddy river banks
- Supplies got wet
- Wagon tipped
- Wagon sank
- Drownings (explicitly mentioned for caulking/float)

Normative contract:
- Crossing resolution is a weighted outcome sample.
- `CROSSING_OUTCOME_WEIGHTS` defines base outcome weights per method.
- Outcome weights are modified by context (depth, swiftness, weather, guide/assistance), and then sampled via
  `rng.crossing()`.
- Outcome effects (loss amounts, time costs, death rules) are policy-defined and emitted as events (satire-ready
  copy keys), but MUST be deterministic given the same state, policy, and RNG seed.

Note (Deluxe log text): the wagon bed is "about 2.5 feet", and deeper fording can swamp the wagon. This is consistent
with the MECC wet-goods threshold at 2.5 ft and swamping past 3.0 ft.

### 9.5 The Dalles endgame: Columbia River rafting vs Barlow Toll Road (Deluxe)

Deluxe introduces a distinct end-of-trail decision at The Dalles:
- “Raft down the River” (Columbia River rafting flow)
- “Take the Barlow Toll Road”

Deluxe evidence:
- OREGON.EXE UI strings include both options and the gating message “You cannot afford to take the Barlow Toll Road.”
- Resource handles exist for a dedicated rafting flow (`RAFTING.CTR`, `RAFTINFO.CTR`, `RAFTING.ANI`).

Normative contract (OTDeluxe90sPolicy):
- This choice is a hard gate at node 16 (The Dalles). The simulation MUST NOT advance miles beyond The Dalles until one
  option is resolved.
- Both options MUST be modeled as deterministic, day-advancing subflows that emit events and can cause losses.
- Policy MUST define:
  - `BARLOW_TOLL_ROAD_COST_CENTS` (cash gating is Deluxe-confirmed; the numeric cost is not yet extracted)
  - `BARLOW_TOLL_ROAD_TIME_DAYS` (time cost)
  - `RAFTING_TIME_DAYS` (time cost, if rafting spans multiple days)
  - `RAFTING_OUTCOME_WEIGHTS` (safe/loss/drown families)

Open Deluxe parity items:
- Exact toll cost in cents.
- Exact rafting/Barlow outcome families, weights, and time costs.

Scope note (Q19/Q20):
- If Dystrail’s current endgame ends earlier (e.g., boss gate) and never reaches The Dalles, implementing the full
  rafting/Barlow tables is out-of-scope for parity v1. Keep the interface and policy parameters stable so it can be
  added later without refactoring the daily kernel.

---

## 10. Random Event System (Oregon Trail Deluxe)

Probability is circumstance-dependent, not fixed.

Normative event selection API:

EventPool = all events
weight(event) = base(event) * F(context)
pick = weighted_choice(EventPool, weight)

Implementation-binding requirement (Q15): event selection MUST produce explainable telemetry
for debugging and empirical fitting:

EventDecisionTrace {
  rolled_u32_or_f32,
  candidates[{id, base_weight, multipliers[], final_weight}],
  chosen_id,
}

Deluxe built-in random events include (as surfaced by Deluxe game text):
- Heavy snow -> wagon becomes snowbound
- Snakebite
- “An Indian helped you find some food.” (legacy phrasing in the original; Dystrail UI copy must be modern/respectful)
- Finding wild fruit
- Blizzard
- Severe storm
- Heavy fog
- Hailstorm
- Strong winds
- Lost trail
- Wrong trail
- Impassable trail
- Rough trail
- Ox wandered off
- Lost party member
- Abandoned wagon (nothing found OR some supplies found)
- Thief stole supplies
- Bad water
- No water
- No grass for the oxen
- Fire in wagon destroyed supplies
- Ox sickness/death
- Wagon part breaks (repairable / replaceable / unrepairable)

Dystrail mapping: encounters + weather incidents + vehicle incidents + exec orders
must collectively cover these families.

---

## 11. Rest, Trade, Hunting (Oregon Trail Deluxe)

Deluxe UX and mechanics:
- Supplies can only be purchased at forts/stores ("You can only buy supplies at forts.").
  - Store stop list is explicit in Deluxe resources (OREGON.EXE @ `0x1EF42`):
    - Independence (store name: “Matt's”)
    - Fort Kearney
    - Fort Laramie
    - Fort Bridger (skipped if Sublette Cutoff is taken)
    - Fort Hall
    - Fort Boise
    - Fort Walla Walla (skipped if the Dalles shortcut is taken)
  - Normative mapping to 8.5 node indices:
    `STORE_NODE_INDICES = [0, 3, 5, 8, 11, 13, 15]`.
- Store economics are deterministic and location-dependent (EXE extracted):
  - Base prices are stored in cents and multiplied by a per-node percentage:
    `price_cents(item, node) = STORE_BASE_PRICE_CENTS[item] * STORE_PRICE_MULT_PCT_BY_NODE[node] / 100`.
  - Base prices and multipliers are listed in 4.3.1 (sources: OREGON.EXE @ `0x1E8DA`).
  - Normative mapping of per-node multipliers (first 18 entries) to the Deluxe trail-node list (8.5):
    - 0 Independence: 100%
    - 1 Kansas River Crossing: 100%
    - 2 Big Blue River Crossing: 100%
    - 3 Fort Kearney: 100%
    - 4 Chimney Rock: 125%
    - 5 Fort Laramie: 125%
    - 6 Independence Rock: 150%
    - 7 South Pass: 150%
    - 8 Fort Bridger: 150%
    - 9 Green River Crossing: 175%
    - 10 Soda Springs: 175%
    - 11 Fort Hall: 175%
    - 12 Snake River Crossing: 200%
    - 13 Fort Boise: 200%
    - 14 Blue Mountains: 225%
    - 15 Fort Walla Walla: 250%
    - 16 The Dalles: 250%
    - 17 Willamette Valley: 250%
  - Store purchase units and hard caps (EXE extracted; OTDeluxe90sPolicy MUST match):
    - Ammo is purchased as `ammo_boxes` and converts to `bullets = ammo_boxes * BULLETS_PER_BOX`.
    - UI/input maximums: oxen ≤ 20, ammo_boxes ≤ 50, clothes_sets ≤ 99, each spare part type ≤ 3, food_lbs ≤ 2000
      (see `STORE_MAX_BUY[...]` in 4.3.1).
- Rest can be 1-9 days (“You may rest for 1 to 9 days.”).
- Trading exists as a discrete action (Deluxe presents trading as a choice at stops/encounters).
  - MECC Appendix B explicitly: each trade costs a day away from the trail; adopt `TRADE_COST_DAYS = 1` for OTDeluxe90sPolicy unless contradicted by Deluxe evidence.
  - Trade UI supports requesting at least: oxen, clothing, bullets, wagon parts (wheel/axle/tongue), food, and cash.
- Hunting exists as a discrete action and is gated:
  - MECC Appendix B explicitly: hunting takes a day away from the trail; adopt `HUNT_COST_DAYS = 1` for OTDeluxe90sPolicy unless contradicted by Deluxe evidence.
  - Hunting is not allowed in some locations (“too many people around”).
  - Hunting is blocked by severe weather (“weather is too severe”).
  - Hunting requires ammunition (“You can't go hunting because you have no bullets.”).
  - Overhunting causes local scarcity (“game will become scarce”).
- Store help provides recommendations (non-binding gameplay guidance, but Deluxe-authored):
  - At least six oxen.
  - At least 200 pounds of food per party member.
  - At least two sets of clothes per party member.
  - Each ammo box contains 20 bullets.
  - Spare parts are recommended because wheels/axles/tongues can break and strand the wagon.
- Wagon capacity constraints (Deluxe):
  - Per-item max holdings are explicit (EXE extracted `STORE_MAX_BUY[...]`), and Deluxe uses wagon-space rejection strings
    (“not enough room in your wagon…”) when a purchase cannot be carried (OREGON.EXE strings @ `0x1EA9A` and `0x1EEED`).
  - OTDeluxe90sPolicy decision (Q10): assume no additional total-weight capacity beyond per-item caps unless proven later.
- Hunting has a carry cap:
  - `hunt_carry_cap_lbs`: maximum food that can be carried back from a hunt in one day.
    - MECC Appendix B states a 100 lb per-carrier limit; Deluxe screenshots strongly support:
      `hunt_carry_cap_lbs = 100 * alive_party_members`.
    - OTDeluxe90sPolicy decision (Q3): injuries do not reduce carry cap; “alive” means “not dead”.

Dystrail mapping:
- Rest/Trade/Hunt are first-class intents; each consumes a day and still runs the daily root-cause
  ticks (Weather -> Supplies burn -> Health/strain) before short-circuiting travel.
- Carry cap and day-costs are OTDeluxe90sPolicy-scoped parity requirements:
  - `trade_cost_days = 1`, `hunt_cost_days = 1`, `rest_days_range = 1..9`,
    with explicit wagon-capacity enforcement (at minimum `STORE_MAX_BUY[...]` per-item caps; plus a total-weight
    capacity model only if Deluxe evidence shows one exists) and `hunt_carry_cap_lbs`.

---

## 12. Endgame Scoring (parity target)

Oregon Trail Deluxe scoring (implementation-binding for `OTDeluxe90sPolicy`):

### 12.1 Per-person points

Deluxe score screen structure indicates per-person points awarded based on the party health
label at arrival (Good/Fair/Poor/Very Poor). The exact point values are not surfaced in the
extracted Deluxe UI strings; therefore they MUST be set explicitly as a policy parameter.

Evidence:
- Deluxe score screen capture confirms at least: “arriving in good health × 500”.
- Still unconfirmed for Deluxe: the point values for Fair/Poor/Very Poor arrivals.

Normative OTDeluxe90sPolicy stance (implementation-binding default; Q16):
- Good is Deluxe-confirmed at 500 points.
- Until Deluxe evidence proves otherwise, all non-Good tiers score 0 points (harsh “OT feel”), and the policy table is
  the only place to change this later.

| Party Health Label | Points per Person (OTDeluxe90sPolicy default) | Evidence |
| ------------------ | --------------------------------------------- | -------- |
| Good | 500 | Deluxe score screen capture |
| Fair | 0 | Default until captured/extracted |
| Poor | 0 | Default until captured/extracted |
| Very Poor | 0 | Default until captured/extracted |

`party_health_label` derivation:
- Deluxe-lineage (health scalar present): derived from `health_general` via the label ranges (see 6.1).
- Dystrail translation (HP/Sanity/Pants authoritative): derived from `general_strain_norm` (see 13.4.1).

### 12.2 Item points

Deluxe score screen strings specify:

| Item | Points |
| ---- | ------ |
| wagon | 50 |
| ox | 4 |
| spare wagon part | 2 |
| set of clothing | 2 |
| bullets (each 50) | 1 |
| food (each 25 lbs) | 1 |
| cash (each $5) | 1 |

### 12.3 Occupation multiplier

Deluxe occupations and final score bonus multipliers (as shown by Deluxe occupation help):

| Occupation | Final Score Bonus |
| ---------- | ----------------- |
| Banker | x 1.0 (none) |
| Doctor | x 1.0 (none) |
| Merchant | x 1.5 |
| Blacksmith | x 2.0 |
| Carpenter | x 2.0 |
| Saddlemaker | x 2.5 |
| Farmer | x 3.0 |
| Teacher | x 3.5 |

### 12.4 Normative formula

points_per_person = SCORE_POINTS_PER_PERSON_BY_HEALTH[party_health_label]   // policy-defined

score =
  (party_alive * points_per_person)
+ (wagons * 50)
+ (oxen * 4)
+ (spare_parts * 2)
+ (clothes * 2)
+ floor(bullets / 50)
+ floor(food_lbs / 25)
+ floor(cash_cents / 500)

score *= occupation_bonus_multiplier   // from occupation table above

Policy note:
- If you capture a Deluxe score screen proving per-person point values that differ from the
  OTDeluxe90sPolicy defaults (Good=500, non-Good=0), OTDeluxe90sPolicy MUST be updated to match the capture.

---

## 13. Dystrail Extensions (Normative Integration)

### 13.1 Vehicle system (replaces oxen)

- vehicle.wear is the mobility health scalar.
- breakdowns are discrete incidents; spares and repairs consume resources/time.
- breakdown probability depends on wear, pace, weather, exec orders, endgame scaling.
- breakdowns can fully block travel (hard-stop).

### 13.2 Exec orders and policies

- Policy overlays define base weights, multipliers, thresholds, and feature toggles.
- Exec orders add temporary modifiers to travel, breakdown chance, encounter chance,
  supplies burn, and strain.

### 13.3 Boss gate and endgame

- Boss gate is a hard stop that blocks travel after daily physics updates.
- Endgame controller scales travel and breakdown risk, and can bias encounter weights.

### 13.4 Derived Scalar: `general_strain` (Dystrail parity for MECC health)

MECC uses `health_general` both as a displayed state and as the input to downstream odds
(illness/injury 0%..40%). Dystrail decomposes "health" into multiple visible axes; therefore
the MECC role MUST be replicated via a derived, hidden scalar computed once per day.

Definition:
- `general_strain` is a non-negative scalar; higher is worse.
- `general_strain` MUST be deterministic given the same state and policy.
- `general_strain` MUST NOT be player-visible (it is a balancing/odds driver).

Normative form (weights are policy-defined):

general_strain =
  w_hp     * (HP_MAX - hp)
+ w_sanity * (SANITY_MAX - sanity)
+ w_pants  * pants
+ w_starve * malnutrition_level
+ w_vehicle* vehicle_wear_norm
+ w_weather* weather_severity
+ w_exec   * exec_order_strain_bonus

Where:
- `vehicle_wear_norm` is a bounded wear signal (e.g., 0..1 or 0..100).
- `weather_severity` is a bounded signal derived from `WeatherEffects`.
- `exec_order_strain_bonus` is the additive strain from active exec orders.

`general_strain` directly feeds ONLY these probability/weighting hooks:
- disease/affliction probability (when the selected policy uses `general_strain` as the odds driver)
- encounter probability deltas and/or event pool weights
- optional travel-block "bad luck" nudges (if the selected policy enables it)

### 13.4.1 Dystrail health-label derivation (UI/scoring parity)

Deluxe uses `health_general` both to drive odds and to display/score the party health label (Good/Fair/Poor/Very Poor).
In Dystrail, HP/Sanity/Pants remain authoritative; therefore we need a deterministic translation from Dystrail state
to a Deluxe-style label for:
- UI parity (status summaries)
- endgame scoring (people-in-good-health points)
- any label-conditioned copy/events that must match Deluxe cadence (without adding RNG)

Normative OTDeluxe90sPolicy mapping (Q18; policy-configurable):

general_strain_norm = clamp(general_strain / STRAIN_NORM_DENOM, 0.0, 1.0)

health_label =
  if general_strain_norm < 0.25: Good
  else if general_strain_norm < 0.50: Fair
  else if general_strain_norm < 0.75: Poor
  else: VeryPoor

Where:
- `STRAIN_NORM_DENOM` is a policy constant that normalizes the typical strain range into 0..1.
- Boundary values are chosen to be monotone and approximately align with the MECC/Deluxe quartile-like health ranges.

### 13.5 Disease/Affliction odds mapping (Deluxe semantics, Dystrail driver)

Deluxe model constraint: daily affliction odds are in [0..0.40] and increase as health worsens.

Affliction odds driver MUST be explicit per policy:
- `OTDeluxe90sPolicy`: odds are derived from `health_general` (Deluxe/series concept).
- Dystrail parity overlays: odds are derived from `general_strain` (Dystrail translation of the same role).

Normative formulas (choose exactly one based on policy):

OTDeluxe90sPolicy:

p_affliction_today =
  clamp(
    affliction_curve_pwl(health_general, policy),   // `AFFLICTION_CURVE_PWL`
    0.0,
    P_AFFLICTION_MAX
  )

Dystrail parity overlays:

p_affliction_today =
  clamp(
    f_strain_to_prob(general_strain, policy),
    0.0,
    P_AFFLICTION_MAX
  )

If triggered:
- choose target (party member or party-wide abstraction) deterministically via `rng.health()`
- choose affliction kind (illness vs injury) via policy weights
- apply duration and daily ticks; emit start/tick/recovery events with satire-ready copy keys

### 13.6 Encounter probability (circumstance-dependent, Deluxe-aligned)

Deluxe model constraint: event probability is circumstance-dependent (not fixed).

Normative daily encounter chance (derived once per day, then rolled once):

encounter_chance_today =
  clamp(
    encounter_base(region, mode, policy)
  + pace_encounter_delta(pace)
  + weather_encounter_delta(weather_today, weather_effects)
  + exec_encounter_delta(exec_orders)
  + strain_encounter_delta(general_strain)
  - cooldown_penalties(recent_history)
  , 0.0, encounter_cap(policy)
  )

Selection is a weighted choice from an encounter pool:

weight(encounter) = base(encounter) * F(context)
pick = weighted_choice(pool, weight, rng.encounter())

Satire requirement: the selected encounter's mechanical payload is stable and deterministic;
only the presentation copy varies by locale/theme.

### 13.7 Travel distance translation (Dystrail distance fields)

Dystrail tracks multiple travel distance values per day; parity requires the semantics be
explicit so that hard-stops behave like MECC.

Normative meanings:
- `distance_today_raw`: the "would have traveled" distance under multipliers, before hard-stops.
- `distance_today`: the "actual" distance after penalties/clamps, before hard-stops.
- `computed_miles_today`: the distance used for endgame/crossing checks and day recording.

Hard-stop rule (Deluxe parity):
- if any hard-stop applies (boss gate, crossing pending, travel_blocked, navigation hard-stop),
  then `computed_miles_today = 0` for that day.

If the implementation keeps Dystrail's existing aggregation (`computed_miles_today = max(distance_today, distance_today_raw)`),
then OTDeluxe90sPolicy MUST ensure `distance_today_raw == distance_today` whenever travel is allowed, so that
"max" cannot leak extra miles across the day boundary.

---

## 14. Normative Daily Pipeline (Dystrail, Deluxe-faithful)

Phase order and allowed mutations:

1) StartOfDay
- resets per-day flags, counters, and daily records
- does NOT consume RNG beyond phase scope

2) WeatherTick (rng.weather)
- select weather and compute WeatherEffects
- update rain/snow accumulators
- set weather-derived modifiers for downstream phases

3) SuppliesBurnTick
- compute and apply supplies burn
- apply starvation backstops if needed

4) HealthTick (rng.health)
- apply baseline recovery/decay
- apply penalties (starvation, disease, weather, exec orders)
- compute general_strain
- tick ally attrition and clamp stats (Dystrail pre-travel checks)
- roll afflictions using the selected policy’s odds driver (see 13.5)

5) BossGateTick
- if boss is ready and not attempted, block travel and return UI gate

6) IntentTick (rest/trade/hunt/continue)
- resolve the player's chosen action for the day
- REST/TRADE/HUNT MUST each consume a full day (progress = 0) and return after recording the day
- CONTINUE proceeds to mobility/travel phases

7) VehicleTick (rng.vehicle/breakdown)
- roll breakdown; resolve and mark travel_blocked if needed
- apply wear penalties

8) TravelBlockTick
- if blocked, record NonTravel day and return

9) EncounterTick (rng.encounter)
- derive encounter_chance_today once per day
- roll for encounter (respect caps and cooldowns)
- on encounter, record partial travel if applicable and return

10) ComputeMilesToday (rng.travel + rng.events)
- compute distance_today_raw and distance_today
- apply navigation hard-stops (lost/wrong/impassable/snowbound) using rng.events (fixed draw order)
- update progress state (miles_traveled / day_record) if travel occurs

11) TravelWearTick
- apply wear for the day's actual travel (0 if hard-stopped or non-travel)

12) EndgameTick
- update endgame controller with computed miles and breakdown events

13) CrossingTick (rng.crossing)
- if crossing reached, block travel and require choice

14) RandomEventTick (rng.events)
- non-navigation events that do not hard-stop travel

15) RecordDay + TerminalChecks
- record day kind and miles
- emit logs/events
- if failure state, end game

---

## 14.1 Phase Ownership (Mutation Boundaries)

To keep the kernel deterministic and auditable, each phase has an explicit "owned" slice of
state. A phase MUST NOT mutate state it does not own (except via emitted events that are
applied within that same phase).

| Phase | Allowed to mutate | Must NOT mutate |
| ----- | ----------------- | --------------- |
| StartOfDay | per-day counters/flags, day record init, decrement timers/cooldowns | weather, travel miles, inventory/resources beyond resets |
| WeatherTick | `weather_state`, precip accumulators, derived `WeatherEffects` | party, vehicle, encounters, endgame |
| SuppliesBurnTick | supplies/resources burn, starvation counters, starvation side-effects | weather selection, encounter selection |
| HealthTick | health/condition state, disease timers, `general_strain` | weather selection, travel miles |
| BossGateTick | boss gating state (if any) | supplies burn, health computation, travel miles |
| IntentTick | action-specific deltas (rest/trade/hunt), intent resolution flags | weather selection, RNG usage outside intent scope |
| VehicleTick | breakdown state, travel_block flags derived from breakdown | travel miles, encounter selection |
| TravelBlockTick | delay counters/credits, day kind tagging | encounter selection, travel miles |
| EncounterTick | encounter selection state, encounter-related partial-day recording | weather selection, supplies burn |
| ComputeMilesToday | distance values, progress counters, hard-stop delays | weather selection, encounter selection |
| TravelWearTick | vehicle wear for the day (based on actual miles) | distance computation for the same day |
| EndgameTick | endgame/boss readiness scaling, victory triggers | core physics deltas (weather/supplies/health/travel) |
| CrossingTick | crossing pending state, crossing outcomes | weather selection, encounter selection |
| RandomEventTick | non-navigation event effects | phase order, RNG outside `rng.events()` |
| RecordDay + TerminalChecks | finalize day record, terminal/victory state | any root-cause physics for the same day |

If a feature needs to affect multiple slices (e.g., weather affecting encounter chance), it MUST
do so by producing a derived effect struct owned by the source phase (e.g., `WeatherEffects`)
that downstream phases read.

---

## 15. Invariants and Determinism Rules

- Weather resolves exactly once per day.
- Supplies burn occurs once per day.
- Health tick occurs once per day after supplies burn.
- Player intent is resolved explicitly once per day; non-travel intents still run daily root-cause ticks.
- Encounter chance is derived once per day and capped.
- Travel hard-stops set miles_today to 0 and block travel for days.
- RNG streams are phase-scoped; no interleaving across phases.
- Logs/events derive from state changes, never drive them.

---

## 16. Policy Selection Checklist (must be explicit)

These MUST be made explicit by selecting a named policy overlay. The spec is internally
consistent within a single overlay; mixing choices across overlays is invalid.

Chosen parity overlay: `OTDeluxe90sPolicy` (Oregon Trail Deluxe ’90s).
`DystrailLegacyPolicy` is optional and out-of-scope for parity unless explicitly selected later.

1) Health recovery baseline
- Resolved (MECC Appendix B; adopted for Deluxe parity unless contradicted by EXE/empirical):
  - `HEALTH_RECOVERY_BASELINE`: additive `health_general -= 10` per day (natural recovery).
  - `HEALTH_LABEL_RANGES`: 0–34 good, 35–69 fair, 70–104 poor, 105–139 very poor.
  - `HEALTH_DEATH_THRESHOLD = 140` (death-imminent).
- Still open for Deluxe specifically: direct confirmation that Deluxe uses the same hidden numeric thresholds.

2) Hunting carry cap
- Resolved (MECC + Deluxe screenshots):
  - MECC Appendix B: 100 lbs per carrier/day.
  - Deluxe screenshots strongly support: `hunt_carry_cap_lbs = 100 * alive_party_members`.
- OTDeluxe90sPolicy decision (Q3): injuries do not reduce carry cap; “alive” means “not dead”.

3) Ferry risk dependency
- Partially resolved (MECC Appendix B; Deluxe confirms cost and presence):
  - `FERRY_MIN_DEPTH_FT = 2.5` (no ferry below this depth).
  - `FERRY_WAIT_DAYS = 0..6` inclusive.
  - OTDeluxe90sPolicy default (Q4): wait-days distribution is uniform over `0..6` (policy-configurable).
  - `FERRY_RISK_RANGE = 0.0..0.10` (risk varies by swiftness; function not specified).
  - `FERRY_COST_CENTS = 500` (Deluxe UI).
- Still open for Deluxe: confirm wait-days distribution; confirm ferry accident outcome families/weights (OTDeluxe90sPolicy default excludes death).

4) Hire guide cost
- Resolved (Deluxe UI text): guide cost is exactly 3 sets of clothing (`GUIDE_COST_CLOTHES = 3`).
- Partially resolved (MECC Appendix B): guide reduces accident risk by 80% (`GUIDE_RISK_MULT = 0.20`).
- OTDeluxe90sPolicy default (Q6): guide also reduces losses on accidents (`GUIDE_LOSS_MULT = 0.50`); still open to fit/confirm the exact Deluxe multiplier.

5) Crossing thresholds and outcomes (Deluxe-authored guidance)
- Partially resolved (MECC Appendix B + Deluxe outcomes):
  - Caulk/float mechanical minimum: `CAULK_FLOAT_MIN_DEPTH_FT = 1.5`.
  - Deluxe caulk/float help recommends attempting only when `depth > 2.5` (`CAULK_FLOAT_HELP_RECOMMENDED_MIN_DEPTH_FT = 2.5`).
  - Fording depth cliffs (MECC explicit):
    - `< 2.5`: low risk (given non-extreme swiftness/banks).
    - `2.5..3.0`: wet goods + `DRYING_COST_DAYS = 1`.
    - `> 3.0`: wagon swamps; losses scale with depth.
  - Outcome families MUST include: safe, stuck_in_mud, supplies_wet, tipped, sank, drownings (Deluxe strings).
- OTDeluxe90sPolicy decision (Q7): swiftness is a continuous risk input (no binary threshold).
- Still open: exact per-method outcome weights; exact loss scaling; the bank-type classifier (muddy/rocky) and its frequency.

6) Wagon capacity model
- Resolved for parity v1 (Deluxe EXE + Deluxe strings):
  - Per-item max holdings exist and are EXE-extracted (`STORE_MAX_BUY[...]`):
    oxen ≤ 20, ammo_boxes ≤ 50, clothes_sets ≤ 99, each spare part type ≤ 3, food_lbs ≤ 2000.
  - Deluxe also emits explicit wagon-space rejection strings (“not enough room in your wagon…”).
  - Therefore, `WAGON_CAPACITY_MODEL = per_item_caps` is the normative default.
- Still open (non-blocking): whether Deluxe ALSO enforces a distinct total-weight capacity model beyond per-item caps.

7) Occupation advantage numerics
- Deluxe lists special advantages (doctor/blacksmith/carpenter/farmer), but does not surface the arithmetic.
- OTDeluxe90sPolicy decision (Q11): implement occupation perks as policy hooks (multipliers/deltas), with defaults that
  reflect the qualitative text:
  - Doctor: reduces “death while sick/injured” checks (`OCC_DOCTOR_FATALITY_MULT`, default 0.50).
  - Blacksmith/Carpenter: increases repair success (`OCC_REPAIR_SUCCESS_MULT`, default 1.25).
  - Farmer: reduces mobility failures (`OCC_MOBILITY_FAILURE_MULT`, default 0.75).
- Still open (non-blocking): fit/confirm the exact Deluxe numerics via Monte Carlo or EXE extraction.

8) Disease/affliction catalog and durations
- Partially resolved (MECC Appendix B + Deluxe named conditions):
  - Core simulation durations (MECC explicit): `ILLNESS_DURATION_DAYS = 10`, `INJURY_DURATION_DAYS = 30`.
  - Repeat selection kills (`AFFLICTION_REPEAT_KILLS = true`).
  - Deluxe surfaces named conditions (cholera/dysentery/measles/typhoid/exhaustion/fever/broken arm/leg/etc.).
    Implement a named catalog for UI/log flavor, but do not assume per-disease durations unless Deluxe evidence contradicts MECC.
- Still open: per-disease incidence weights; per-disease stat effects beyond “health worsens”.

9) Minimum mobility requirement (oxen/vehicle)
- Deluxe strings include hard blocks for “no oxen” and “only ox is sick”.
- Resolved (MECC Appendix B + Deluxe strings):
  - `OXEN_MIN_TO_MOVE = 1.0` effective oxen.
  - `OXEN_MIN_FOR_BASE = 4`.
  - `SICK_OX_WEIGHT = 0.5`.
  - Speed scaling when `< 4`: multiply by `(effective_oxen / 4)`.

10) Climate tables and daily weather arithmetic
- OTDeluxe90sPolicy default (Q13): use `DystrailRegionalWeather` (`weather.json` weights) as the active generator.
- Still open (optional future): Deluxe-exact station tables/sampling procedure (`OTDeluxeStationsWeather`) and precip thresholds.

11) Base travel rate and terrain/season multipliers
- Mostly resolved (MECC Appendix B + Deluxe EXE mile markers):
  - `BASE_MPD_PLAINS_STEADY_GOOD = 20`.
  - `PACE_MULT = [1.0, 1.5, 2.0]`.
  - `TERRAIN_MULT[mountains] = 0.5`.
  - Sick-member speed penalty is 10% per sick member.
- Still open: snow-depth slowdown curve (`M_snow(snow_depth)`) and any additional season/region multipliers Deluxe applies.

12) Event pool weights and delay durations
- Deluxe requires circumstance-dependent event probabilities, but does not surface the exact base weights.
  Policy MUST define:
  - base weights per event family
  - context multipliers `F(context)` per family
  - delay-day distributions for navigation hard-stops (lost/wrong/impassable/snowbound)

13) Endgame per-person point values
- Deluxe score UI shows the structure (people × health_label × points), but not the per-person point values.
  Policy MUST set `SCORE_POINTS_PER_PERSON_BY_HEALTH`.
- Resolved for v1 (Q16):
  - Good = 500 (Deluxe-confirmed).
  - Fair/Poor/Very Poor = 0 by default until Deluxe evidence proves otherwise.

14) Death-imminent grace days
- Deluxe phrasing is "within a few days". Policy MUST set `DEATH_IMMINENT_GRACE_DAYS` and its reset behavior.

15) Mapping Deluxe health to Dystrail-visible stats
- Dystrail exposes HP/Sanity/Pants while Deluxe scoring and many odds are defined in terms of a single
  health label (Good/Fair/Poor/Very Poor).
- Resolved (Dystrail normative decision):
  - Authoritative state: HP/Sanity/Pants (and other Dystrail stats).
  - Derived scalar: `general_strain` (computed once per day).
  - Usage: `general_strain` drives probabilities and context multipliers (afflictions, encounter risk nudges, navigation-block nudges).
  - Prohibition: never derive HP/Sanity/Pants from `general_strain` (no hidden feedback loops).
  - Scoring labels: use `general_strain_norm` mapping (13.4.1) with policy constants `STRAIN_NORM_DENOM` and `STRAIN_LABEL_BOUNDS`.

16) The Dalles endgame gate (Columbia River rafting vs Barlow Toll Road)
- Deluxe-confirmed:
  - Both choices exist in Deluxe UI strings (“Raft down the River”, “Take the Barlow Toll Road”).
  - Cash gating exists for Barlow (“You cannot afford to take the Barlow Toll Road.”).
  - Dedicated rafting resources exist (`RAFTING.CTR`, `RAFTINFO.CTR`, `RAFTING.ANI`).
- Still open: numeric toll cost, time cost, and the full deterministic outcome model (families + weights + loss semantics) for both options.
- Scope note (Q19/Q20): treat this as out-of-scope for parity v1 unless Dystrail is implementing a Dalles-equivalent “final route choice” node.

17) Affliction probability curve (general health -> 0%..40% daily odds)
- Resolved for v1 (Q2): `AFFLICTION_CURVE_PWL` (piecewise-linear monotone, clamps at 0.40).
- Still open (non-blocking): empirically fit the curve to Deluxe runs and update policy without kernel changes.

Resolve remaining items by locking OTDeluxe90sPolicy parameters in a single place (no per-system overrides),
and do not implement until each item is explicitly specified or intentionally deferred.
