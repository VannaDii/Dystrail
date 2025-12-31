# ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md

## Normative Systems Specification for Dystrail (MECC-Model Derived, Modernized)

STATUS: AUTHORITATIVE / IMPLEMENTATION-BINDING AUDIENCE: Codex, senior engineers, simulation designers SCOPE: Entire core journey simulation loop

---

## 0. Intent and Non-Negotiables

This document defines the _authoritative_ simulation model for Dystrail. It is not descriptive, illustrative, or suggestive.

All rules, ordering constraints, formulas, thresholds, and invariants herein MUST be implemented exactly unless explicitly overridden by a named policy layer.

Primary goals:

- Preserve the **causal structure** of the MECC Oregon Trail model
- Translate semantics into **modern Dystrail concepts**
- Remove historical artifacts (wagon, oxen) without altering dynamics
- Be deterministic, replayable, and auditable

---

## 1. Conceptual Translation (OT → Dystrail)

| MECC / Oregon Trail    | Role                          | Dystrail Equivalent                |
| ---------------------- | ----------------------------- | ---------------------------------- |
| Wagon                  | Mobility + inventory platform | Vehicle (wear, breakdowns, spares) |
| Oxen                   | Speed & failure gating        | Vehicle readiness & wear           |
| Food (lbs)             | Daily consumption             | Supplies                           |
| Clothing               | Cold mitigation, trade        | Gear tags + trade currency         |
| Bullets                | Hunting input                 | Ammo / hunting resource            |
| General Health (0–140) | Collapse accumulator          | Derived General Strain             |
| Individual illness     | Per-person risk               | Party conditions                   |
| Pace                   | Speed vs risk                 | PaceId (steady/heated/blitz)       |
| Rations                | Health vs burn                | DietId (quiet/mixed/doom)          |
| Random events          | Circumstance-driven           | Encounters + weather + exec orders |
| Rivers                 | Hard gates                    | Crossings                          |
| Forts                  | Safe stops                    | Camps / stores                     |
| Score                  | Post-run eval                 | Endgame scoring                    |

---

## 2. Canonical State Model

### 2.1 Persistent State (Required)

The following state MUST exist and persist across days.

- day: u32
- miles_traveled: f32
- region: enum { Heartland, RustBelt, Beltway }
- mode: enum { Classic, Deep }
- policy: enum { Balanced, Conservative, Aggressive, ResourceManager }

### Party & Stats

- supplies: i32
- hp: i32
- sanity: i32
- morale: i32
- credibility: i32
- allies: i32
- pants: i32
- budget: i32

### Vehicle

- wear: f32
- tolerance: f32
- breakdown_state: enum { Healthy, Degraded, Broken }
- spares: struct

### Systems

- weather: struct { today, rain_accum, snow_accum }
- exec_orders: struct { active, modifiers }
- encounters: struct { cooldowns, caps, history }
- travel_state: struct { blocked, delay_days, partial_travel }
- boss_state: struct { ready, active, resolved }
- endgame: struct { scaling, thresholds }

---

## 3. Derived Scalar: General Strain

General Strain replaces MECC's general health accumulator. It MUST be recomputed once per day, after health consumption and penalties.

Formula (normative):

general_strain = w_hp \* (HP_MAX - hp)

- w_sanity \* (SANITY_MAX - sanity)
- w_pants \* pants
- w_supply \* starvation_level
- w_vehicle\* normalized_vehicle_wear
- w_weather\* weather_severity
- w_exec \* exec_order_strain_bonus

Constraints:

- Lower is better
- No upper bound
- Must be deterministic
- Must NOT be player-visible

General Strain directly feeds:

- Disease probability
- Encounter probability deltas
- Travel failure deltas

---

## 4. Weather System (Root Cause)

Weather is the FIRST causal system each day.

Ordering constraint: Weather MUST resolve before:

- supplies burn
- health decay/recovery
- vehicle breakdown rolls
- encounter probability derivation
- crossing risk

Weather produces a WeatherEffects struct:

- travel_speed_factor
- supplies_drain_delta
- sanity_delta
- pants_delta
- encounter_delta
- breakdown_delta
- crossing_risk_delta

Weather accumulation (rain/snow) MUST persist and decay over time.

---

## 5. Supplies Burn System

Supplies burn occurs EVERY day, regardless of travel.

Formula:

supplies_burn = base_burn(region)

- pace_factor(pace)
- weather_factor(weather)
- vehicle_factor(vehicle_state)
- exec_factor(exec_orders)
- diet_factor(diet)

Apply: supplies -= supplies_burn

Starvation triggers:

- escalating hp loss
- escalating sanity loss
- encounter weighting changes

---

## 6. Health & Collapse

### 6.1 Baseline Tick

Each day applies baseline effects:

hp += hp_recovery(weather, rest_state) sanity += sanity_recovery(weather, rest_state) pants += pants_baseline(pace, diet)

Then apply penalties:

- starvation
- disease ticks
- weather severity
- executive orders
- vehicle stress

### 6.2 Disease Roll

Disease probability derives ONLY from general_strain:

p_disease = clamp(f(general_strain), MIN_DISEASE, MAX_DISEASE)

If triggered:

- select disease (mode/policy weighted)
- apply duration
- tick daily penalties
- resolve to recovery or death

---

## 7. Travel & Progress

Raw distance:

distance_today_raw = BASE_DISTANCE(mode)

- pace_multiplier(pace)
- weather_multiplier(weather)
- vehicle_multiplier(vehicle)
- exec_multiplier(exec_orders)
- starvation_multiplier

Hard stops override ALL distance:

- travel_blocked
- crossing_pending
- boss_gate

Final distance_today = 0 if blocked else distance_today_raw

---

## 8. Vehicle System

Vehicle replaces wagon/oxen mechanics.

Vehicle wear:

- increases daily with travel
- scales with pace, weather, exec orders

Breakdown probability depends on:

- wear
- pace
- weather
- exec orders
- endgame scaling
- policy overlays

Breakdowns:

- may fully block travel
- require repair actions
- consume supplies, budget, or time

---

## 9. Encounters

Encounter chance MUST be derived ONCE per day:

encounter_chance = base(region, mode)

- pace_delta
- weather_delta
- exec_delta
- strain_delta

* cooldown_penalty

Constraints:

- Max 1 encounter/day unless explicitly overridden
- Cooldowns must be enforced

---

## 10. Crossings

Crossings are HARD GATES.

Upon reaching a crossing:

- travel halts
- player must choose resolution

Choices:

- detour
- bribe
- permit

Each choice defines:

- cost
- success chance
- failure penalties
- days lost

---

## 11. Terminal Conditions

Immediate failure if:

- hp <= 0
- sanity <= 0
- pants >= threshold
- boss failure state

Victory if:

- miles_traveled >= victory_distance
- boss resolved (if required)

---

## 12. Invariants

- One weather resolution per day
- One supplies burn per day
- One health tick per day
- RNG must be phase-scoped
- Logs derive from events, never drive logic
