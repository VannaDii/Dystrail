# ENGINE_JOURNEY_CONTROLLER_DIFF.md

## Mapping Current Dystrail JourneyController to Parity Kernel

---

### Current travel_next_leg Order

1. start_of_day
2. guard_boss_gate
3. pre_travel_checks
4. vehicle_roll / resolve_breakdown
5. handle_travel_block
6. process_encounter_flow
7. apply_travel_wear
8. endgame controller
9. handle_crossing_event
10. record_day + end_of_day

---

### Required Parity Order

1. start_of_day
2. WeatherTick
3. SuppliesBurnTick
4. HealthTick (compute general_strain)
5. guard_boss_gate
6. VehicleTick
7. handle_travel_block
8. EncounterTick (chance derived once)
9. TravelWearTick
10. EndgameTick
11. CrossingTick
12. record_day + end_of_day

---

### Mandatory Refactors

- Extract weather logic into explicit WeatherTick
- Move supplies burn before health
- Introduce GeneralStrain computation
- Derive encounter chance once per day
- Enforce phase-scoped RNG usage

---

### Codex Instructions

- Do not inline logic across phases
- Each phase owns its state mutations
- All changes must preserve determinism
- No UI coupling inside kernel
