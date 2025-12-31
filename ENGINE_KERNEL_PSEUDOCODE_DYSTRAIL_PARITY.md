# ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md

## Normative Daily Tick Kernel (Implementation Binding)

---

function DAILY_TICK(state, cfg, rng):

StartOfDay(state)

// Phase 1: Weather weather = ResolveWeather(state, rng.weather) ApplyWeatherEffects(state, weather)

// Phase 2: Supplies Burn burn = ComputeSuppliesBurn(state) state.supplies -= burn Emit(Event.SuppliesConsumed, burn)

// Phase 3: Health ApplyBaselineRecovery(state) ApplyHealthPenalties(state) general_strain = ComputeGeneralStrain(state)

if RollDisease(general_strain, rng.health): ApplyDisease(state)

// Phase 4: Boss Gate if BossGateBlocks(state): Emit(Event.BossAwait) EndOfDay(state) return

// Phase 5: Vehicle if RollBreakdown(state, rng.vehicle): ResolveBreakdown(state)

if VehicleBlocksTravel(state): RecordNonTravelDay(state, reason="vehicle") EndOfDay(state) return

// Phase 6: Encounters encounter_chance = ComputeEncounterChance(state, general_strain) if rng.encounter.roll(encounter_chance): enc = SelectEncounter(state, rng.encounter) ApplyEncounter(state, enc) EndOfDay(state) return

// Phase 7: Travel distance = ComputeDistanceToday(state) state.miles_traveled += distance ApplyTravelWear(state)

// Phase 8: Endgame RunEndgameController(state)

// Phase 9: Crossings if CrossingReached(state): Emit(Event.CrossingPending) EndOfDay(state) return

// Phase 10: Terminal Checks if TerminalFailure(state): Emit(Event.GameOver) EndOfDay(state) return

EndOfDay(state)
