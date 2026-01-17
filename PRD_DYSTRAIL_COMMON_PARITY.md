# PRD_DYSTRAIL_COMMON_PARITY.md

## Scope and method

Dystrail sources: ENGINE_JOURNEY_CONTROLLER_DIFF.md, ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md, ENGINE_OVERHAUL_CHECKLIST.md, ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md.
External repos compared: [Maxwolf/OregonTrail@01ff8d1](https://github.com/Maxwolf/OregonTrail/tree/01ff8d185dbbe7681ac126e75a32638f93c2f7c9); [clintmoyer/oregon-trail@582a5b9](https://github.com/clintmoyer/oregon-trail/tree/582a5b996298fc2bf521df84abc6d0240d5e08fb)
Focus: mechanics and outcomes only (not code structure).
Parity target for the Dystrail docs: Oregon Trail Deluxe DOS v3.0 with explicit policy overlays for any divergence.
Evidence links refer to the source code online at the pinned commit.
Legend:
- Parity status: Yes (matches), Partial (some overlap, key deltas), No (missing or contradictory), Unknown (not found).
- Repo agreement: Agree (same mechanic), Partial (same family but different rules), Diverge (different or one missing).

## 0. Current Dystrail pipeline reference (Journey diff)

**Spec detail**
Current travel_next_leg order is StartOfDay (reset flags/counters, apply starvation tick, roll illness, apply sanity guard, process weather, clamp stats, apply travel wear scaled), BossGate, pre-travel checks, vehicle roll/breakdown, travel block, encounter flow, travel wear, endgame controller, crossing event, record travel day and end-of-day.

**Maxwolf parity**
No - daily order is Time->Trail; Trail ticks location then vehicle; vehicle ticks people, sets mileage, and triggers events. No explicit boss gate, pre-travel checks, encounter flow, or endgame controller phases.

**Evidence (Maxwolf)**
- [src/Module/Time/TimeModule.cs#L92-L143](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Time/TimeModule.cs#L92-L143)
- [src/Module/Trail/TrailModule.cs#L116-L135](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/TrailModule.cs#L116-L135)
- [src/Entity/Location/Location.cs#L192-L200](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Location/Location.cs#L192-L200)
- [src/Entity/Vehicle/Vehicle.cs#L444-L473](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L444-L473)

**Clintmoyer parity**
No - two week turn loop with fort/hunt/continue and a single event phase; no daily pipeline phases.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L221-L250](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L221-L250)
- [freebasic/oregon.bas#L294-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L294-L307)

**Repo agreement**
Diverge - Maxwolf has daily ticks; Clintmoyer uses two week turns.

## 1. Required OTDeluxe parity day order (Journey diff, Systems spec 14, Kernel pseudocode)

**Spec detail**
- Day is the atomic step producing DayInputs, DayEffects, DayEvents, and DayRecord with deterministic replay
- required order is StartOfDay, WeatherTick, SuppliesBurnTick, HealthTick, BossGateTick, IntentTick, VehicleTick, TravelBlockTick, EncounterTick, ComputeMilesToday, TravelWearTick, EndgameTick, CrossingTick, RandomEventTick, RecordDay + TerminalChecks + EndOfDay.

**Maxwolf parity**
No - phases are not explicit; weather tick and person/vehicle ticks are intertwined, and events are triggered during vehicle or crossing ticks.

**Evidence (Maxwolf)**
- [src/GameSimulationApp.cs#L101-L114](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/GameSimulationApp.cs#L101-L114)
- [src/Module/Time/TimeModule.cs#L92-L143](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Time/TimeModule.cs#L92-L143)
- [src/Module/Trail/TrailModule.cs#L116-L123](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/TrailModule.cs#L116-L123)
- [src/Entity/Person/Person.cs#L227-L262](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L227-L262)

**Clintmoyer parity**
No - two week turn cadence and event selection embedded in the turn; no daily phases.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L294-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L294-L307)
- [freebasic/oregon.bas#L399-L414](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L399-L414)

**Repo agreement**
Diverge - different simulation cadence and phase structure.

## 2. Phase mapping matrix deltas coverage (Journey diff)

**Spec detail**
- StartOfDay should not include weather/illness ordering conflicts
- WeatherTick must be before starvation/illness
- SuppliesBurnTick must exist and precede HealthTick
- HealthTick should compute general_strain
- BossGateTick after weather/supplies/health
- IntentTick must be explicit
- VehicleTick must be after WeatherTick
- TravelBlockTick must record non-travel and delays
- EncounterTick must be single-source
- ComputeMilesToday explicit
- TravelWearTick after ComputeMilesToday
- EndgameTick uses computed miles
- CrossingTick hard-stops
- RandomEventTick separate from navigation
- RecordDay + Terminal.

**Maxwolf parity**
No - no explicit phase ownership or ordering; supplies and illness are checked inside Person.OnTick before food consumption, weather is ticked separately, and vehicle tick triggers events directly.

**Evidence (Maxwolf)**
- [src/Entity/Person/Person.cs#L240-L262](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L240-L262)
- [src/Entity/Person/Person.cs#L268-L286](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L268-L286)
- [src/Entity/Location/Location.cs#L192-L200](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Location/Location.cs#L192-L200)
- [src/Entity/Vehicle/Vehicle.cs#L444-L466](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L444-L466)

**Clintmoyer parity**
No - two week turn and event list; no distinct phase ordering or explicit supplies burn tick.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L294-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L294-L307)
- [freebasic/oregon.bas#L399-L414](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L399-L414)

**Repo agreement**
Diverge.

## 3. Mandatory deltas for OTDeluxe parity (Journey diff)

**Spec detail**
1. Weather is root cause and precedes supplies/health/vehicle/encounters.
2. Explicit SuppliesBurnTick.
3. General health scalar (health_general or general_strain) is authoritative.
4. Affliction probability maps from health/strain via curve.
5. Navigation hard-stops set miles_today=0 and delays.
6. Trail mile markers and shortcuts match Deluxe tables (Sublette and Dalles).
7. Rest/Trade/Hunt are intent-based non-travel days.
8. Encounter chance derived once per day.
9. RNG consumption phase-scoped.

**Maxwolf parity**
Partial - has weather and travel events and rest/trade/hunt consume time, but lacks weather root-cause fan-out, explicit supplies burn tick, health_general or general_strain, affliction curve, Deluxe mile tables, single-source encounter chance, and phase-scoped RNG.

**Evidence (Maxwolf)**
- [src/Entity/Location/Weather/LocationWeather.cs#L101-L153](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Location/Weather/LocationWeather.cs#L101-L153)
- [src/Entity/Person/Person.cs#L346-L372](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L346-L372)
- [src/Module/Trail/Trail.cs#L84-L128](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/Trail.cs#L84-L128)
- [src/Module/Director/EventDirectorModule.cs#L57-L73](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Director/EventDirectorModule.cs#L57-L73)
- [src/Window/Travel/Trade/Trading.cs#L169-L173](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Trade/Trading.cs#L169-L173)

**Clintmoyer parity**
No - two week turns, no health_general, no supplies burn tick, no Deluxe mile tables, no intent system, and single RNG.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L294-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L294-L307)
- [freebasic/oregon.bas#L399-L414](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L399-L414)
- [freebasic/oregon.bas#L721-L736](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L721-L736)

**Repo agreement**
Diverge.

## 4. What already matches OTDeluxe well (Journey diff)

**Spec detail**
- single daily entrypoint
- domain RNG bundle
- vehicle breakdown logic
- encounter flow analogous to random events
- partial travel wear scaling.

**Maxwolf parity**
Partial - daily entrypoint exists (Time->Trail), breakdown events exist, random events exist, but no domain RNG bundle and no explicit wear scaling.

**Evidence (Maxwolf)**
- [src/Module/Time/TimeModule.cs#L92-L143](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Time/TimeModule.cs#L92-L143)
- [src/Module/Director/EventDirectorModule.cs#L57-L73](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Director/EventDirectorModule.cs#L57-L73)
- [src/Event/Vehicle/BrokenVehiclePart.cs#L26-L53](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Vehicle/BrokenVehiclePart.cs#L26-L53)

**Clintmoyer parity**
No - no daily entrypoint, no RNG bundle, no vehicle breakdown system beyond event list.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L221-L250](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L221-L250)
- [freebasic/oregon.bas#L399-L418](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L399-L418)

**Repo agreement**
Diverge.

## 5. Political satire alignment (Journey diff, Systems spec 2.1, Kernel pseudocode)

**Spec detail**
- mechanics must be value-neutral
- events are mechanical kinds
- satire is presentation-only using i18n keys
- satire adds no RNG or timing changes
- optional flavor must be deterministic from the same event payload
- satire targets systems not protected traits.

**Maxwolf parity**
No - narrative strings and mechanics are interwoven in code, no i18n key separation or satire layer.

**Evidence (Maxwolf)**
- [src/Event/Vehicle/LostTrail.cs#L35-L38](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Vehicle/LostTrail.cs#L35-L38)
- [src/Event/Vehicle/VehicleFire.cs#L39-L43](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Vehicle/VehicleFire.cs#L39-L43)
- [src/Event/Wild/Thief.cs#L45-L49](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Wild/Thief.cs#L45-L49)
- [src/Window/RandomEvent/RandomEvent.cs#L59-L67](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/RandomEvent/RandomEvent.cs#L59-L67)

**Clintmoyer parity**
No - narrative strings embedded in logic, no i18n separation.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L414-L520](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L414-L520)

**Repo agreement**
Agree - both embed narrative directly and do not separate satire or i18n.

## 6. RNG streams and determinism (Kernel pseudocode, Systems spec 4.1, 15)

**Spec detail**
- deterministic RNG bundle with per-domain streams (weather, health, travel, events, crossing, trade, hunt, vehicle/breakdown, encounter)
- phases may only consume their listed streams with fixed draw order
- no cross-phase draws
- early returns still finalize day
- events derive from state and do not drive logic.

**Maxwolf parity**
No - single Random instance used across weather, health, vehicle, and events; no phase-scoped RNG.

**Evidence (Maxwolf)**
- [src/Entity/Location/Weather/LocationWeather.cs#L107-L133](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Location/Weather/LocationWeather.cs#L107-L133)
- [src/Entity/Person/Person.cs#L346-L372](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L346-L372)
- [src/Entity/Vehicle/Vehicle.cs#L226-L242](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L226-L242)
- [src/Module/Director/EventDirectorModule.cs#L57-L65](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Director/EventDirectorModule.cs#L57-L65)

**Clintmoyer parity**
No - single RND source seeded once, used across travel, events, and illness.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L22-L24](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L22-L24)
- [freebasic/oregon.bas#L307-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L307-L307)
- [freebasic/oregon.bas#L399-L403](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L399-L403)
- [freebasic/oregon.bas#L721-L728](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L721-L728)

**Repo agreement**
Agree - both use a single RNG without stream separation.

## 7. Event bus structure (Kernel pseudocode, Systems spec 4.2)

**Spec detail**
- Event { id, day, kind, severity, payload, tags, ui_surface_hint } with events as logs only
- UI uses event payloads
- mechanics are not driven by event rendering.

**Maxwolf parity**
No - events are executed as game windows and drive state changes.

**Evidence (Maxwolf)**
- [src/Module/Director/EventDirectorModule.cs#L94-L103](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Director/EventDirectorModule.cs#L94-L103)
- [src/Window/RandomEvent/RandomEvent.cs#L59-L67](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/RandomEvent/RandomEvent.cs#L59-L67)
- [src/Event/Vehicle/OxenDied.cs#L27-L41](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Vehicle/OxenDied.cs#L27-L41)

**Clintmoyer parity**
No - no event bus, events are inline code blocks.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L399-L520](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L399-L520)

**Repo agreement**
Agree - neither uses an event bus.

## 8. Policy overlays and explicit policy selection (Kernel pseudocode, Systems spec 4.3, 16, Overhaul checklist)

**Spec detail**
- named policy overlays (OTDeluxe90sPolicy, DystrailLegacyPolicy) with explicit parameters
- no mixing across overlays
- PolicySet includes base_weights, multipliers, thresholds, feature_toggles, per_region_overrides, per_season_overrides.

**Maxwolf parity**
No - core parameters are hard-coded (event roll chance, river costs, pace/ration enums) with no policy overlay or explicit parameter sets.

**Evidence (Maxwolf)**
- [src/Module/Director/EventDirectorModule.cs#L57-L63](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Director/EventDirectorModule.cs#L57-L63)
- [src/Window/Travel/RiverCrossing/RiverGenerator.cs#L39-L55](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/RiverCrossing/RiverGenerator.cs#L39-L55)
- [src/Entity/Vehicle/TravelPace.cs#L9-L24](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/TravelPace.cs#L9-L24)
- [src/Entity/Person/RationLevel.cs#L11-L27](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/RationLevel.cs#L11-L27)

**Clintmoyer parity**
No - mechanics are encoded with inline constants; no policy overlay system.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L295-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L295-L307)
- [freebasic/oregon.bas#L721-L730](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L721-L730)

**Repo agreement**
Agree - neither has policy overlays.

## 9. Required state model (Systems spec 3.1 and 3.2)

**Spec detail**
Required MECC day state fields (day, miles_traveled, region, season, party, party_alive, health_general, oxen_healthy, food_lbs, bullets, clothes, cash_cents, spares, pace, rations, weather_today, snow_depth, rain_accum, river_state, wagon_state, flags/mods) plus Dystrail-specific state (stats with hp/sanity/morale/credibility/allies/pants/budget, vehicle wear, mode, policy, encounters struct, exec_orders, travel flags, general_strain, boss, endgame, weather_state).

**Maxwolf parity**
Partial - has party, odometer, inventory (food/ammo/clothes/cash), pace, rations, weather condition, and vehicle status, but lacks health_general, oxen_healthy, cash_cents, snow/rain accumulation, and Dystrail-specific state.

**Evidence (Maxwolf)**
- [src/Entity/Vehicle/Vehicle.cs#L100-L139](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L100-L139)
- [src/Entity/Vehicle/Vehicle.cs#L200-L214](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L200-L214)
- [src/Entity/Location/Location.cs#L53-L67](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Location/Location.cs#L53-L67)
- [src/Entity/Person/HealthStatus.cs#L12-L39](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/HealthStatus.cs#L12-L39)

**Clintmoyer parity**
Partial - has variables for miles, food, ammo, clothes, cash, and misc supplies but no structured state or health_general.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L216-L219](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L216-L219)
- [freebasic/oregon.bas#L741-L770](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L741-L770)

**Repo agreement**
Partial - both track supplies and miles but not the required structured state model.

## 10. Weather system (Systems spec 5, Journey diff delta #1)

**Spec detail**
- weather is the root cause
- labels are rainy/snowy/very rainy/very snowy or temp bands (very hot >90F, hot 70-90F, warm 50-70F, cool 30-50F, cold 10-30F, very cold <10F)
- maintain rain_accum and snow_depth with evaporation and melt
- WeatherEffects include travel_speed_factor, supplies_drain_delta, sanity_delta, pants_delta, encounter_chance_delta, breakdown_chance_delta, crossing_risk_delta, river_depth_delta
- DystrailRegionalWeather (weather.json) is default but must preserve causal fan-out, and OTDeluxeStationsWeather is optional with required TEMP_MEAN/RAIN_MEAN tables plus sampling variance and precip thresholds.

**Maxwolf parity**
No - weather is ticked per location but does not drive a causal fan-out into health, supplies, travel, or event weights; no rain/snow accumulation.

**Evidence (Maxwolf)**
- [src/Entity/Location/Location.cs#L192-L200](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Location/Location.cs#L192-L200)
- [src/Entity/Location/Weather/LocationWeather.cs#L101-L153](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Location/Weather/LocationWeather.cs#L101-L153)

**Clintmoyer parity**
No - weather appears only as event text (heavy rain, cold weather, blizzard) with no explicit weather model or accumulation.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L437-L453](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L437-L453)
- [freebasic/oregon.bas#L568-L574](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L568-L574)

**Repo agreement**
Agree - neither implements the required weather causal model.

## 11. Health system and affliction model (Systems spec 6, Journey diff deltas #3 and #4, Kernel pseudocode)

**Spec detail**
- health_general scalar (0 best, higher worse, no hard max) with label ranges 0-34 good, 35-69 fair, 70-104 poor, 105-139 very poor, >=140 death imminent
- daily update includes baseline recovery of -10 and additive penalties (weather, pace, rations, clothing, afflictions, drought, event effects)
- p_affliction_today = clamp(AFFLICTION_CURVE_PWL(health_general), 0..0.40) with PWL segments 0..34 -> 0.05, 35..69 -> 0.15, 70..104 -> 0.25, 105..139 -> 0.40
- durations 10 days illness, 30 days injury
- repeat selection kills
- disease catalog with optional fatality model and doctor perk
- affliction selection and fatality rolls use rng.health
- sick party speed penalty 10 percent per sick member
- death-imminent countdown with grace days (default 3) and reset_on_recovery_below_threshold as default reset mode.

**Maxwolf parity**
No - uses per-person HealthStatus enum without health_general, PWL affliction curve, or death-imminent countdown; illness checks use ration/clothes and random rolls.

**Evidence (Maxwolf)**
- [src/Entity/Person/HealthStatus.cs#L12-L39](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/HealthStatus.cs#L12-L39)
- [src/Entity/Person/Person.cs#L240-L258](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L240-L258)
- [src/Entity/Person/Person.cs#L346-L372](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L346-L372)

**Clintmoyer parity**
No - illness routine depends on eating and clothing, no scalar or durations, and no death-imminent countdown.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L438-L446](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L438-L446)
- [freebasic/oregon.bas#L721-L736](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L721-L736)

**Repo agreement**
Agree - neither implements the required health_general and affliction curve model.

## 12. Supplies burn and rations (Systems spec 7, Journey diff delta #2)

**Spec detail**
- rations (filling/meager/bare bones) affect consumption and health and bare bones can harm health
- supplies burn formula includes base burn and modifiers for pace, weather, vehicle, exec orders, and diet
- starvation mechanics when supplies <= 0
- supplies burn occurs even on non-travel intents.

**Maxwolf parity**
Partial - rations exist (Filling/Meager/Bare Bones) and drive food consumption, but there is no explicit supplies burn formula or weather/pace modifiers; illness is checked before food consumption.

**Evidence (Maxwolf)**
- [src/Entity/Person/RationLevel.cs#L11-L27](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/RationLevel.cs#L11-L27)
- [src/Entity/Person/Person.cs#L240-L262](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L240-L262)
- [src/Entity/Person/Person.cs#L268-L286](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L268-L286)

**Clintmoyer parity**
Partial - rations are chosen each turn (poor/moderate/well) and reduce food by a fixed formula, but no per-day supplies burn formula or modifiers.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L295-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L295-L307)

**Repo agreement**
Partial - both model ration choices but not the required burn and modifier pipeline.

## 13. Travel viability gates and base speed model (Systems spec 8.0 to 8.3)

**Spec detail**
- travel is blocked if effective oxen < 1.0 (sick ox counts as 0.5)
- base miles per day is 20 on plains steady with >=4 effective oxen
- multipliers for pace (1.0/1.5/2.0) aligned to 8/12/16 hour help text and strenuous/grueling health risk, terrain (mountains 0.5), oxen scaling, sick party (-10 percent per sick), and snow
- miles_today = base * multipliers + random event adjustment.

**Maxwolf parity**
No - travel uses RandomMileage based on oxen cost, random halving, and event reductions; pace does not affect mileage; oxen health and sick party speed penalty are not modeled.

**Evidence (Maxwolf)**
- [src/Entity/Vehicle/Vehicle.cs#L226-L242](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L226-L242)
- [src/Entity/Vehicle/Vehicle.cs#L458-L473](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L458-L473)
- [src/Entity/Vehicle/TravelPace.cs#L9-L24](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/TravelPace.cs#L9-L24)

**Clintmoyer parity**
No - progress per turn is M = M + 200 + (A - 220) / 5 + 10 * RND; no base 20 miles per day model, no pace multipliers, no oxen health model.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L102-L110](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L102-L110)
- [freebasic/oregon.bas#L307-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L307-L307)

**Repo agreement**
Diverge.

## 14. Hard-stop semantics for navigation events (Systems spec 8.4, Journey diff delta #5)

**Spec detail**
- lost/wrong/impassable/snowbound events set miles_today = 0 and add multi-day delays
- navigation hard-stops are distinct from other random events.

**Maxwolf parity**
Partial - LostTrail, WrongTrail, and ImpassableTrail exist and use LoseTime days skipped, but there is no explicit miles_today concept or snowbound event.

**Evidence (Maxwolf)**
- [src/Event/Vehicle/LostTrail.cs#L22-L38](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Vehicle/LostTrail.cs#L22-L38)
- [src/Event/Vehicle/WrongTrail.cs#L23-L38](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Vehicle/WrongTrail.cs#L23-L38)
- [src/Event/Vehicle/ImpassableTrail.cs#L23-L38](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Vehicle/ImpassableTrail.cs#L23-L38)
- [src/Event/Prefab/LoseTime.cs#L24-L43](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Prefab/LoseTime.cs#L24-L43)

**Clintmoyer parity**
Partial - mountains section includes lost trail and blizzard events that reduce mileage, but no explicit miles_today=0 semantics.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L538-L540](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L538-L540)
- [freebasic/oregon.bas#L568-L574](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L568-L574)

**Repo agreement**
Partial - both include some hard-stop style events but not the Deluxe semantics.

## 15. Trail graph, mile markers, and shortcuts (Systems spec 8.5, Journey diff delta #6, Kernel pseudocode TrailGraph)

**Spec detail**
fixed node order and mile markers (main route) with total miles 2083, Oregon City is presentation-only; Sublette Cutoff and Dalles shortcut route variants with 0 sentinels and explicit mile savings; Dalles gate at node 16. Main route markers:

| Node index | Location | Mile marker |
| --- | --- | --- |
| 0 | Independence, Missouri | 0 |
| 1 | Kansas River Crossing | 102 |
| 2 | Big Blue River Crossing | 185 |
| 3 | Fort Kearney | 304 |
| 4 | Chimney Rock | 554 |
| 5 | Fort Laramie | 640 |
| 6 | Independence Rock | 830 |
| 7 | South Pass | 932 |
| 8 | Fort Bridger | 989 |
| 9 | Green River Crossing | 1151 |
| 10 | Soda Springs | 1295 |
| 11 | Fort Hall | 1352 |
| 12 | Snake River Crossing | 1534 |
| 13 | Fort Boise | 1648 |
| 14 | Grande Ronde (Blue Mountains) | 1808 |
| 15 | Fort Walla Walla | 1863 |
| 16 | The Dalles | 1983 |
| 17 | Willamette Valley | 2083 |

Sublette Cutoff: South Pass to Green River = 125, skip Fort Bridger (0 sentinel), savings 94, total 1989.
Dalles shortcut: Blue Mountains to The Dalles = 125, skip Fort Walla Walla (0 sentinel), savings 50, total 2033.
Combined: skip Fort Bridger and Fort Walla Walla, savings 144, The Dalles 1839, Willamette 1939.

**Maxwolf parity**
Partial - node list and fork choices exist (South Pass to Green River shortcut, Dalles fork), but distances are random and Oregon City is modeled as a node rather than presentation-only.

**Evidence (Maxwolf)**
- [src/Module/Trail/TrailRegistry.cs#L24-L53](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/TrailRegistry.cs#L24-L53)
- [src/Module/Trail/Trail.cs#L84-L128](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/Trail.cs#L84-L128)

**Clintmoyer parity**
No - no node list or mile marker tables; total distance is 2040 and progression is a single mileage counter.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L32-L35](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L32-L35)
- [freebasic/oregon.bas#L307-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L307-L307)
- [freebasic/oregon.bas#L632-L633](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L632-L633)

**Repo agreement**
Diverge.

## 16. River crossing system (Systems spec 9, Journey diff checklist)

**Spec detail**
- Rivers modeled (Kansas, Big Blue, Green, Snake, Columbia)
- depth/width/swiftness derived from rainfall/season
- options and constraints with specific thresholds: ferry cost $5, min depth 2.5, wait 0..6 days with uniform default, risk range 0.0..0.10 and nonlethal accident default
- guide cost 3 clothes with risk mult 0.20 and loss mult 0.50
- caulk/float min depth 1.5 and recommended >2.5
- ford recommended max depth 2.5, wet goods in 2.5..3.0 with 1 drying day and no permanent loss by default, swamps >3.0
- swiftness is a continuous risk input
- outcome weights are policy-defined and outcome families include safe, stuck_in_mud, supplies_wet, tipped, sank, drownings
- crossing cost days 1
- Dalles gate requires rafting vs Barlow Toll Road with policy-defined costs/time/outcomes.

**Maxwolf parity**
Partial - river crossings exist with ferry/float/ford/guide options, but river depth/width are random (not rainfall driven), ferry cost is random 3-8 with wait 1-10 days, guide cost random 3-8 clothes, and depth thresholds differ (ford washout >3, float flood >5). Dalles toll cost is random.

**Evidence (Maxwolf)**
- [src/Window/Travel/RiverCrossing/RiverGenerator.cs#L29-L55](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/RiverCrossing/RiverGenerator.cs#L29-L55)
- [src/Window/Travel/RiverCrossing/CrossingTick.cs#L213-L239](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/RiverCrossing/CrossingTick.cs#L213-L239)
- [src/Event/River/VehicleWashOut.cs#L14-L46](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/River/VehicleWashOut.cs#L14-L46)
- [src/Window/Travel/Toll/TollGenerator.cs#L19-L32](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Toll/TollGenerator.cs#L19-L32)

**Clintmoyer parity**
No - no crossing system or choices; only a random event about fording.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L492-L495](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L492-L495)

**Repo agreement**
Diverge.

## 17. Encounter and random event selection (Systems spec 10, Journey diff delta #8)

**Spec detail**
- encounter/event chance is context-dependent and derived once per day as clamp(encounter_base + pace delta + weather delta + exec delta + strain delta - cooldown penalties, 0..cap)
- selection is weighted by base * F(context) and produces an EventDecisionTrace with candidates, multipliers, and chosen id
- event families include snowbound, snakebite, helpful Indians, fruit, blizzard, storms, fog, hail, strong winds, lost/wrong/impassable, rough trail, ox wandered, lost party member, abandoned wagon, thief, bad/no water, no grass, fire, ox sickness/death, and wagon part breaks.

**Maxwolf parity**
Partial - event categories exist and include many families, but selection is a flat 1 percent roll per category with no telemetry or context weights; some families are missing (no grass/no water/rough trail/snowbound).

**Evidence (Maxwolf)**
- [src/Module/Director/EventDirectorModule.cs#L57-L73](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Director/EventDirectorModule.cs#L57-L73)
- [src/Event/EventCategory.cs#L10-L40](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/EventCategory.cs#L10-L40)
- [src/Event/Wild/Thief.cs#L14-L35](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Wild/Thief.cs#L14-L35)
- [src/Event/Vehicle/VehicleFire.cs#L13-L43](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Vehicle/VehicleFire.cs#L13-L43)

**Clintmoyer parity**
Partial - event selection uses fixed thresholds (EventData) and includes some families (wagon breaks, ox injures, lost child, unsafe water, heavy rains/cold, bandits, fire, fog, snakebite, swamped river, wild animals, hail, illness) but no context weighting or telemetry and missing several Deluxe families.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L399-L412](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L399-L412)
- [freebasic/oregon.bas#L414-L520](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L414-L520)

**Repo agreement**
Partial - both implement random events but not the Deluxe weighted model.

## 18. Rest, trade, and hunting intents (Systems spec 11, Journey diff delta #7)

**Spec detail**
- rest is 1-9 days
- trade and hunt each cost 1 day
- non-travel intents still run daily root-cause ticks
- hunting requires ammo, is blocked by severe weather and some locations, has overhunting scarcity, and carry cap is 100 * alive_party_members with injuries not reducing the cap
- trade supports oxen, clothing, bullets, parts, food, cash.

**Maxwolf parity**
Partial - rest range is 1-9 days, trade and hunt consume time via TakeTurn(false), hunting requires ammo and is allowed only on the trail; no severe-weather gating or scarcity; carry cap is fixed at 100 lbs (not scaled by party).

**Evidence (Maxwolf)**
- [src/Window/Travel/Rest/RestAmount.cs#L48-L51](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Rest/RestAmount.cs#L48-L51)
- [src/Window/Travel/Rest/Resting.cs#L82-L89](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Rest/Resting.cs#L82-L89)
- [src/Window/Travel/Trade/Trading.cs#L169-L173](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Trade/Trading.cs#L169-L173)
- [src/Window/Travel/Hunt/HuntManager.cs#L42-L47](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Hunt/HuntManager.cs#L42-L47)
- [src/Window/Travel/Hunt/Help/NoAmmo.cs#L35-L39](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Hunt/Help/NoAmmo.cs#L35-L39)

**Clintmoyer parity**
Partial - hunting requires bullets and consumes time (M = M - 45), but there is no explicit rest or trade intent system.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L232-L241](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L232-L241)
- [freebasic/oregon.bas#L270-L283](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L270-L283)

**Repo agreement**
Partial - both include hunting with ammo gating, but other intent mechanics diverge.

## 19. Store economy and inventory constraints (Systems spec 4.3.1 and 11, Journey diff checklist)

**Spec detail**
- buy only at forts
- store node indices [0,3,5,8,11,13,15]
- cash in cents
- bullets per box 20
- base prices in cents (ox 2000, clothes 1000, bullet 10, ammo box 200 where 200 = 10 * 20, food lb 20, wheel/axle/tongue 1000)
- per-node price multipliers list [100,100,100,100,125,125,150,150,150,175,175,175,200,200,225,250,250,250,250]
- store max buy caps (oxen 20, ammo boxes 50, clothes 99, spare parts 3 each, food 2000)
- per-item caps enforced
- wagon capacity model uses per-item caps with optional total-weight capacity only if proven
- store help recommends 6 oxen, 200 lbs food per person, 2 clothes per person, ammo boxes 20 bullets.

**Maxwolf parity**
Partial - stores exist at settlements; base costs match for ox ($20), clothes ($10), food ($0.20), parts ($10), ammo ($2) but no per-node multipliers (costs are static), cash is float dollars, ammo is boxes but 20 bullets per box is not enforced explicitly, and caps differ (clothes max 50, ammo max 99). Per-item caps for oxen (20), parts (3), food (2000) align.

**Evidence (Maxwolf)**
- [src/Entity/Item/Resources.cs#L15-L26](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Item/Resources.cs#L15-L26)
- [src/Entity/Item/Parts.cs#L15-L33](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Item/Parts.cs#L15-L33)
- [src/Entity/Item/SimItem.cs#L81-L93](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Item/SimItem.cs#L81-L93)
- [src/Window/Travel/Store/StorePurchase.cs#L57-L69](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Store/StorePurchase.cs#L57-L69)
- [src/Entity/Vehicle/Vehicle.cs#L125-L139](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L125-L139)
- [src/Entity/Location/Point/Settlement.cs#L27-L32](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Location/Point/Settlement.cs#L27-L32)

**Clintmoyer parity**
No - fort purchases are based on dollars spent and yield 2/3 of that in items (implicit price markup), no per-node multipliers, no explicit caps, and ammo is 50 bullets per $1.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L49-L51](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L49-L51)
- [freebasic/oregon.bas#L252-L266](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L252-L266)

**Repo agreement**
Diverge.

## 20. Occupations and advantages (Systems spec 4.3.1 and 12.3)

**Spec detail**
- occupations list is [banker, doctor, merchant, blacksmith, carpenter, saddlemaker, farmer, teacher] with starting cash [1600,1200,1200,800,800,800,400,400] and bonus multipliers [1.0,1.0,1.5,2.0,2.0,2.5,3.0,3.5]
- doctor reduces fatality odds, blacksmith/carpenter improve repairs, farmer reduces mobility failures.

**Maxwolf parity**
Partial - only banker, carpenter, farmer exist with starting monies 1600/800/400 and score multipliers 1x/2x/3x; no doctor/merchant/blacksmith/saddlemaker/teacher or their perks.

**Evidence (Maxwolf)**
- [src/Entity/Person/Profession.cs#L11-L26](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Profession.cs#L11-L26)
- [src/Window/MainMenu/Profession/ProfessionSelector.cs#L113-L133](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/MainMenu/Profession/ProfessionSelector.cs#L113-L133)
- [src/Window/GameOver/FinalPoints.cs#L132-L160](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/GameOver/FinalPoints.cs#L132-L160)

**Clintmoyer parity**
No - no occupation system.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L81-L86](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L81-L86)

**Repo agreement**
Diverge.

## 21. Scoring model (Systems spec 12, Journey diff checklist)

**Spec detail**
score per person by health label (Good=500, others 0 by default), item points (wagon 50, ox 4, spare part 2, clothes 2, bullets/50, food/25, cash/$5), and occupation multiplier applied at end.

**Maxwolf parity**
Partial - item points follow similar divisors (ammo points per 50, food per 25, cash per 5) and occupation multiplier exists, but per-person points use HealthStatus numeric values (Good 500, Fair 400, Poor 300, VeryPoor 200) rather than Good-only scoring; only 3 occupations are supported.

**Evidence (Maxwolf)**
- [src/Window/GameOver/FinalPoints.cs#L84-L168](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/GameOver/FinalPoints.cs#L84-L168)
- [src/Entity/Person/HealthStatus.cs#L17-L33](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/HealthStatus.cs#L17-L33)
- [src/Entity/Item/Resources.cs#L15-L26](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Item/Resources.cs#L15-L26)

**Clintmoyer parity**
No - no scoring system, only arrival text and remaining supplies.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L628-L686](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L628-L686)

**Repo agreement**
Diverge.

## 22. Dystrail extensions (Systems spec 13, Kernel pseudocode)

**Spec detail**
- vehicle wear as mobility health, breakdown probability depends on wear/pace/weather/exec orders
- exec orders and policies modify travel, breakdown, encounter, supplies, strain
- boss gate is a hard stop after daily physics
- general_strain derived daily and not player-visible using weights for hp/sanity/pants/starvation/vehicle wear/weather/exec orders
- general_strain_norm = clamp(general_strain / STRAIN_NORM_DENOM, 0..1) with label bounds 0.25/0.50/0.75
- affliction odds driver is explicit per policy (health_general in OTDeluxe90sPolicy or general_strain in Dystrail overlays)
- encounter chance derived once per day with cooldowns
- travel distance translation includes distance_today_raw, distance_today, computed_miles_today with hard-stop rule and no miles leak (distance_today_raw == distance_today for OTDeluxe policy).

**Maxwolf parity**
No - no wear or exec orders, no boss gate, no general_strain, and no distance_today_raw vs computed_miles_today semantics.

**Evidence (Maxwolf)**
- [src/Entity/Vehicle/Vehicle.cs#L100-L125](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L100-L125)
- [src/Entity/Person/HealthStatus.cs#L12-L39](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/HealthStatus.cs#L12-L39)

**Clintmoyer parity**
No - none of these concepts present.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L741-L770](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L741-L770)

**Repo agreement**
Agree - neither implements Dystrail extensions.

## 23. Phase ownership and invariants (Systems spec 14.1 and 15)

**Spec detail**
- each phase has explicit state ownership (weather only mutates weather_state, supplies burn only resources, etc)
- invariants include weather once per day, supplies burn once per day, health tick after supplies, encounter chance derived once per day, hard-stops set miles to 0, phase-scoped RNG.

**Maxwolf parity**
No - no explicit phase ownership, and weather/supplies/health are interleaved in per-entity ticks.

**Evidence (Maxwolf)**
- [src/Module/Trail/TrailModule.cs#L116-L123](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/TrailModule.cs#L116-L123)
- [src/Entity/Person/Person.cs#L240-L262](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L240-L262)
- [src/Entity/Vehicle/Vehicle.cs#L444-L466](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L444-L466)

**Clintmoyer parity**
No - no phase ownership; eating, travel, events, and mountains are in the same turn flow.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L294-L575](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L294-L575)

**Repo agreement**
Agree.

## 24. Overhaul checklist locked decisions (ENGINE_OVERHAUL_CHECKLIST.md)

**Spec detail**
- Oregon City is presentation-only
- store multiplier schedule is normative
- affliction curve is the PWL curve
- hunting carry cap is 100 * alive
- ferry defaults include min depth 2.5, wait days 0..6 uniform, ferry accidents non-lethal by default
- Snake River guide defaults (risk mult 0.20, loss mult 0.50)
- swiftness is continuous
- ford wet goods depth 2.5..3.0 with 1 drying day
- wagon capacity per-item caps only
- weather generator remains Dystrail regional but must preserve causal fan-out and snow accumulators remain present even if dormant
- arrival scoring defaults Good=500, others 0
- death-imminent grace days default 3 with reset on recovery
- occupation defaults for doctor/repair/mobility multipliers (0.50/1.25/0.75).

**Maxwolf parity**
No - Oregon City is a node, store multipliers not implemented, affliction curve not present, hunting carry cap fixed at 100, ferry and guide costs are randomized, swiftness and depth cliffs are different, and no death-imminent or occupation perk multipliers are present.

**Evidence (Maxwolf)**
- [src/Module/Trail/TrailRegistry.cs#L24-L53](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/TrailRegistry.cs#L24-L53)
- [src/Window/Travel/RiverCrossing/RiverGenerator.cs#L39-L55](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/RiverCrossing/RiverGenerator.cs#L39-L55)
- [src/Window/Travel/Hunt/HuntManager.cs#L42-L47](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Hunt/HuntManager.cs#L42-L47)
- [src/Entity/Item/SimItem.cs#L315-L320](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Item/SimItem.cs#L315-L320)

**Clintmoyer parity**
No - none of these locked decisions exist in the 1978 code.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L31-L66](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L31-L66)
- [freebasic/oregon.bas#L492-L495](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L492-L495)

**Repo agreement**
Agree.

## 25. Overhaul checklist implementation order and coverage index

**Spec detail**
checklist includes ordered implementation steps (lock decisions, policy/RNG/events/phase boundaries, kernel cutover, state model/migrations, trail graph, daily ticks, economy/store, scoring, UI, determinism/tests) and a coverage index mapping MUST requirements to checklist IDs.

**Maxwolf parity**
No - no equivalent checklist or structured implementation plan in repo.

**Evidence (Maxwolf)**
- [README.md#L1-L107](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/README.md#L1-L107)

**Clintmoyer parity**
No - no equivalent checklist or mapping.

**Evidence (Clintmoyer)**
- [README.md#L1-L31](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/README.md#L1-L31)

**Repo agreement**
Agree.

## 26. Dystrail kernel pseudocode specifics (ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md)

**Spec detail**
- DayContext fields (day, region, season, pace, rations, inventory, weather_state, vehicle_state, exec_orders, policy, mode, general_strain)
- DayOutcome fields (day_record, events, travel_kind, ui_state, ended)
- TrailGraph has mile markers per route variant with 0 sentinels, store node indices, and store pricing multipliers indexed by node order
- derivations require route variant selection, current_node_index derived from miles and markers, and 0 sentinel handling.

**Maxwolf parity**
No - no DayContext/DayOutcome structures or TrailGraph sentinel logic; trail distances are randomized and not derived from mile markers.

**Evidence (Maxwolf)**
- [src/Module/Trail/Trail.cs#L84-L128](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/Trail.cs#L84-L128)
- [src/GameSimulationApp.cs#L101-L114](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/GameSimulationApp.cs#L101-L114)

**Clintmoyer parity**
No - no structured context/outcome or TrailGraph.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L294-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L294-L307)
- [freebasic/oregon.bas#L741-L770](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L741-L770)

**Repo agreement**
Agree.

## 27. TICK_DAY kernel phases (ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md)

**Spec detail**
explicit StartOfDay, Weather, Consumption, Health, GeneralStrain, PreTravel checks, Affliction roll, BossGate, Ferry wait, Intent (Rest/Trade/Hunt), Vehicle breakdowns, Travel blocks, Encounter chance, Compute miles, Navigation hard-stops, Travel wear, Endgame, Crossing/Landmark gating, Random events, Terminal checks with deterministic RNG scoping.

**Maxwolf parity**
No - no explicit daily kernel; travel and events occur during vehicle and crossing ticks; no general_strain or affliction roll phase.

**Evidence (Maxwolf)**
- [src/Module/Time/TimeModule.cs#L92-L143](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Time/TimeModule.cs#L92-L143)
- [src/Module/Trail/TrailModule.cs#L116-L123](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/TrailModule.cs#L116-L123)
- [src/Entity/Vehicle/Vehicle.cs#L444-L466](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Vehicle/Vehicle.cs#L444-L466)
- [src/Entity/Person/Person.cs#L227-L262](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Person/Person.cs#L227-L262)

**Clintmoyer parity**
No - no daily kernel; two week turns.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L294-L307](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L294-L307)
- [freebasic/oregon.bas#L399-L414](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L399-L414)

**Repo agreement**
Agree.

## 28. Crossing resolver details (ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md)

**Spec detail**
- choice availability gates (ferry min depth 2.5, caulk min depth 1.5), ford wet goods 2.5..3.0 with drying day, swamping past 3.0 with severe outcomes, caulk/float guidance >2.5, ferry cost $5 with wait-days drawn before outcome, guide cost 3 clothes with risk reduction, outcome families include safe/stuck/wet/tipped/sank/drown
- RNG stream crossing only.

**Maxwolf parity**
Partial - gates and outcomes exist but with different thresholds and random costs/delays, and RNG is global.

**Evidence (Maxwolf)**
- [src/Window/Travel/RiverCrossing/RiverGenerator.cs#L29-L55](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/RiverCrossing/RiverGenerator.cs#L29-L55)
- [src/Window/Travel/RiverCrossing/CrossingTick.cs#L213-L239](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/RiverCrossing/CrossingTick.cs#L213-L239)
- [src/Event/River/SuppliesWet.cs#L13-L37](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/River/SuppliesWet.cs#L13-L37)

**Clintmoyer parity**
No - no crossing resolver.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L492-L495](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L492-L495)

**Repo agreement**
Diverge.

## 29. Dalles endgame resolver (ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md)

**Spec detail**
- The Dalles gate blocks travel until rafting or Barlow Toll Road is resolved
- Barlow requires cash and advances days
- policy must define BARLOW_TOLL_ROAD_COST_CENTS, BARLOW_TOLL_ROAD_TIME_DAYS, RAFTING_TIME_DAYS, and RAFTING_OUTCOME_WEIGHTS
- rafting and Barlow outcomes are deterministic
- gate is at node 16.

**Maxwolf parity**
Partial - Dalles fork exists with Columbia River or Barlow Toll Road, but toll cost is randomized and there is no explicit day-advancing resolver or policy parameters.

**Evidence (Maxwolf)**
- [src/Module/Trail/TrailRegistry.cs#L43-L50](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/TrailRegistry.cs#L43-L50)
- [src/Window/Travel/Toll/TollGenerator.cs#L19-L32](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Toll/TollGenerator.cs#L19-L32)

**Clintmoyer parity**
No - no Dalles gate.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L31-L35](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L31-L35)

**Repo agreement**
Diverge.

## 30. Deluxe-specific parity checklist items (Journey diff checklist)

**Spec detail**
- occupation bonus multiplier applied to final score
- ferry cost $5 and guide cost 3 clothes
- ford vs caulk/float thresholds
- hunting gating by location and severe weather and carry caps
- stores only at forts with ammo 20 bullets per box and caps (oxen <=20, ammo boxes <=50, clothes <=99, parts <=3, food <=2000) and per-node price multipliers
- trail progression matches Deluxe mile markers and shortcuts
- Dalles gate optional if endgame stops earlier.

**Maxwolf parity**
Partial - occupation multipliers exist but only 3 professions; ferry/guide costs and thresholds differ; hunting gating by location and ammo but not severe weather; store caps partly match but clothes and ammo caps differ; no price multipliers; trail markers are random; Dalles gate exists.

**Evidence (Maxwolf)**
- [src/Window/GameOver/FinalPoints.cs#L132-L160](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/GameOver/FinalPoints.cs#L132-L160)
- [src/Window/Travel/RiverCrossing/RiverGenerator.cs#L39-L55](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/RiverCrossing/RiverGenerator.cs#L39-L55)
- [src/Window/Travel/Hunt/HuntManager.cs#L42-L47](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Window/Travel/Hunt/HuntManager.cs#L42-L47)
- [src/Entity/Item/Resources.cs#L15-L26](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Item/Resources.cs#L15-L26)
- [src/Module/Trail/Trail.cs#L84-L128](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Module/Trail/Trail.cs#L84-L128)

**Clintmoyer parity**
No - no Deluxe store caps or mile markers; no Dalles gate; ferry/guide mechanics absent.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L32-L35](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L32-L35)
- [freebasic/oregon.bas#L252-L266](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L252-L266)
- [freebasic/oregon.bas#L492-L495](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L492-L495)

**Repo agreement**
Diverge.

## 31. Terminology mapping (Systems spec 2)

**Spec detail**
MECC to Dystrail mapping includes Wagon -> Vehicle, Oxen -> Vehicle readiness or wear proxy, Food -> Supplies, Clothing -> Gear tags and trade currency, Bullets -> Ammo resource, General Health -> general_strain + HP/Sanity/Pants outputs, Illness/Injury -> party conditions, Pace -> PaceId, Rations -> DietId, Random events -> Encounters + exec orders + weather incidents, Rivers -> Crossings, Forts -> Stops/camps/stores, Score -> Endgame scoring.

**Maxwolf parity**
Partial - uses classic wagon/oxen/food/clothes/bullets terms but no Dystrail mapping to supplies or general_strain, and no exec orders or gear tags.

**Evidence (Maxwolf)**
- [src/Entity/Entities.cs#L12-L74](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Entities.cs#L12-L74)
- [src/Entity/Item/Resources.cs#L15-L26](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Entity/Item/Resources.cs#L15-L26)

**Clintmoyer parity**
No - original terminology only, no Dystrail mapping.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L741-L770](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L741-L770)

**Repo agreement**
Partial - both use original terms rather than Dystrail translations.

## 32. Satire mapping examples (Systems spec 2.1)

**Spec detail**
- example mapping of MECC event families to Dystrail satire wrappers includes Bad water -> infrastructure neglect, Inadequate grass -> supply chain shock, Lost/Wrong/Lost trail -> red tape detour, Thief at night -> lobbyist raid, Indians help find food -> local mutual aid, Gravesite -> news cycle memorial, Fire in wagon -> equipment or PR fire
- wrapper may rename but must keep mechanical event kind and payload.

**Maxwolf parity**
No - no satire layer or wrapper mapping.

**Evidence (Maxwolf)**
- [src/Event/Wild/Thief.cs#L45-L49](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Wild/Thief.cs#L45-L49)
- [src/Event/Vehicle/VehicleFire.cs#L39-L43](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Vehicle/VehicleFire.cs#L39-L43)
- [src/Event/Vehicle/LostTrail.cs#L35-L38](https://github.com/Maxwolf/OregonTrail/blob/01ff8d185dbbe7681ac126e75a32638f93c2f7c9/src/Event/Vehicle/LostTrail.cs#L35-L38)

**Clintmoyer parity**
No - no satire layer or wrapper mapping.

**Evidence (Clintmoyer)**
- [freebasic/oregon.bas#L414-L520](https://github.com/clintmoyer/oregon-trail/blob/582a5b996298fc2bf521df84abc6d0240d5e08fb/freebasic/oregon.bas#L414-L520)

**Repo agreement**
Agree.
