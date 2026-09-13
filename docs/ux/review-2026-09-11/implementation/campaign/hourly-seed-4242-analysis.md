# Hourly pacing: seed 4242 and the remaining experience guard

Date: 2026-09-12. This is an analysis and proposed measurement plan. No game configuration, automated player strategy, acceptance threshold, or snapshot baseline was changed for this audit. No Rust build or test was run by this audit while the main agent's Clippy checks were running.

## Current runtime evidence

`/tmp/dystrail-hourly-native-07.log` reports 154 passing engine unit tests and 57 passing web library tests. The new clock, stationary-action, encounter-ride, crossing-wear, and UI tests pass. Four assertions remain failing:

1. `journey_config_snapshot_stable`: configuration snapshot drift.
2. `deterministic_journey_digest_baseline`: journey ledger digest drift.
3. `deterministic_csv_digest_for_fixed_seed`: canonical CSV digest drift.
4. `balanced_run_survives_past_day_45`: `expected survival past day 20, got 3`.

The first three are stored configuration/replay baselines. Their replacements should wait until intentional mechanics changes and the experience campaign have been validated. The fourth is an experience failure. Its name says day 45, but its actual assertion requires `days_survived >= 20`; preserve that assertion and the existing strategy.

The current failure log prints the day, not the terminal cause, final inventory, or crossing telemetry. A fresh verbose seed-4242 run is still needed after the coordinated build freeze ends to record those details. The older `/tmp/dystrail-seed4242-current.csv` is dated 13:36 and predates the hourly changes. It records a Classic Balanced crossing collapse on day 40 at 656.2 internal progress miles, with one crossing, one failure, no permit, and no bribe. That is historical evidence, not a fresh observation of the day-three failure.

## The fixed-seed test uses the real campaign action path

In `dystrail-tester/src/logic/game_tester.rs`, the test calls `build_tester()`, which creates `GameTester` with `TesterAssets::load_default()`. Those assets come from the actual web data directory: encounters, personas, store, pacing, weather, camp, boss, and endgame data. Its plan is Classic/Balanced with a 55-day maximum, no custom setup, and seed 4242.

The real-game campaign uses the same `run_plan` and `SimulationSession::advance` implementation. Its full-game plan raises the maximum to 200 days and attaches expectations evaluated after the run. Neither difference explains a termination on day three.

Both paths apply the Staffer persona, the existing Balanced store plan, six crew members, and the same player policy. Before driving, the simulation resolves pending crew care, an encounter, an explicitly selected affordable repair, town activities, gathering, and any requested camp action. Town visits claim the local conversation, perform a three-hour paid/supply shift, buy rations and missing tire/battery stock, then depart. Crew care costs one hour; encounters cost 30 minutes; repair choices use `RepairChoice::minutes()`; gathering costs two hours. The simulation uses `interactive_repairs = true` and the same `choose_repair`, `perform_activity`, `apply_choice`, and `advance_clock` mechanics used by the browser. The browser supplies player choices and outcome screens; the tester supplies its existing policy choices.

The browser also defaults to the Balanced journey configuration. The fixed-seed test is not bypassing town income or the current repair system, and should not be replaced with an easier synthetic setup.

## Static crossing derivation, separate from runtime observation

The engine gives crossings an isolated RNG stream. `derive_stream_seed` in `dystrail-game/src/journey/mod.rs` takes the first eight little-endian bytes of HMAC-SHA256, keyed by the user's seed bytes and containing the domain tag `crossing`. For seed 4242 this produces stream seed `1539624600688737240`.

Applying the installed `rand 0.8.8` native `SmallRng` initialization and first-draw formulas gives sample `4231103604`, a ratio of approximately **0.985130576**. This was calculated from the source formulas in a read-only Python command, not obtained from a new engine run.

The Balanced crossing overlay has pass/detour/terminal weights 0.66/0.24/0.10. `crossings/resolver.rs` normalizes the weights and compares that sample with their cumulative thresholds:

| First-crossing state | Terminal starts at approximately | Result for the native first draw |
| --- | ---: | --- |
| No permit, no bribe | 0.900000000 | Terminal failure |
| No permit, first bribe | 0.952048823 | Terminal failure |
| Applicable permit | 1.000000000 | No terminal failure |

The first bribe adds 0.192 to pass and subtracts 0.045 from terminal before normalization; it cannot save this draw. The first checkpoint therefore kills this native seed **if no permit has been acquired**, regardless of its cash balance. The Balanced initial and town shopping plans do not buy a press pass. Town receipt rewards are named `local-record:<stop>` and do not contain the required `permit` or `press_pass` tags. An encounter could still grant protection, so the current runtime inventory must be recorded before declaring the complete day-three trace proven.

Travel, weather, and encounter changes cannot shift this isolated first crossing draw. A shorter day may allow additional encounters before reaching it, but slowing the calendar alone does not directly change its outcome. This makes a first-checkpoint collapse the strong explanation for the observed day-three failure, rather than a missing town/work path.

## Pacing and early terminal risk are two separate problems

Staffer's physical route is **2,956.273 road miles**. The Balanced finish remains 2,400 internal progress units. The first crossing at 650 units is therefore about **800.657 road miles** from Sacramento, after Reno and Salt Lake City.

Stretching that first crossing to 20 days through speed/time tuning alone would imply roughly 40 road miles per day. That would miss the user's original modern-travel bands of 100–200 miles for leisurely travel, 200–300 for moderate travel, and 300+ for fast travel. A full trip near 150 road miles per day could legitimately take around 20 days, but the early terminal risk also needs attention.

The earlier `/tmp/dystrail-hourly-04.csv` provides a useful, explicitly older baseline: Classic Balanced finishers averaged 13.42 days (range 12–15); Deep Balanced finishers averaged 13.88 (12–17). None of the 250 runs in either group lasted 20 days. Those numbers precede the final clock corrections and must be remeasured. They show that merely saving the first crossing would usually leave the fixed-seed guard unsatisfied because the whole trip finishes too soon.

## Recommended measured global sweep

After the current visual/depth preview is verified and refreshed, capture a fresh unchanged-config seed trace and campaign baseline. Then evaluate **4-, 5-, and 6-hour active travel windows** against the current eight-hour baseline, preserving the 60/70/80 MPH caps, the real hours charged for other actions, the existing player policies, and all pinned experience guards. These are candidate measurements, not selected values.

| Available window | Steady clear-road ceiling | Heated clear-road ceiling | Blitz clear-road ceiling |
| --- | ---: | ---: | ---: |
| 4 hours | 240 miles | 280 miles | 320 miles |
| 5 hours | 300 miles | 350 miles | 400 miles |
| 6 hours | 360 miles | 420 miles | 480 miles |
| Current 8 hours | 480 miles | 560 miles | 640 miles |

Actual daily distances must be measured after weather, vehicle condition, encounters, work, gathering, and town arrivals. A town conversation plus a shift uses 3.5 hours, leaving only half an hour in a four-hour window if performed at the beginning of the day. Short windows can therefore introduce stopped days and violate the travel-day ratio guard even when clear-road speed looks appropriate.

Changing `TRAVEL_DAY_MINUTES` is also not a pure distance adjustment. It divides wear and the probability exposure calculation. At six hours, unchanged daily coefficients produce approximately 33% more wear intensity per driving hour than at eight hours; at five hours, 60% more; at four hours, twice as much. More calendar days additionally change food, diet, health, weather exposure, cooldowns, and crew illness. Record and retune those consequences deliberately.

Alongside that sweep, use globally applicable early-checkpoint consequences that preserve a playable early journey, then restore meaningful late-trip failure pressure. Do not target seed 4242 with a special case or choose weights solely to put its sample just below a threshold. Early setbacks can cost real time/resources; terminal risk should be evaluated against the intended stage and preparedness. Current Balanced boss reach was already excessive in hourly-04, so making every crossing universally harmless without compensating late-trip balance would worsen another pinned expectation.

Encounter variety needs a separate correction. In hourly-04, Classic Balanced had four zero-encounter runs and Deep Balanced had six; successful runs could also miss the minimum variety requirement. A daily cooldown combined with hourly exposure rolls leaves too few opportunities in shortened journeys. Measure encounter spacing in actual travel time and test a bounded early drought/rotation mechanism across the full seed set. Changing the minimum variety guard or the tester's choices is not an acceptable substitute.

For every candidate, retain the 20-day fixed-seed assertion and record physical miles, driving minutes, total days, moving-day ratio, encounters and unique encounters, crossing outcomes, boss reach/wins, survival, failure-family distribution, final resources, and day-ledger consistency. Re-evaluate every pinned band, not only the first reported failure. Once a candidate meets the experience expectations and passes native/Wasm/browser verification, refresh the preview for that completed unit and then regenerate intentional snapshot digests.

## Cross-target replay caveat

The action mechanics are shared, but exact seed trajectories currently differ by compilation target. The installed `rand 0.8.8` chooses Xoshiro256++ for 64-bit `SmallRng` and Xoshiro128++ for 32-bit `SmallRng`, including wasm32. The same static derivation gives the first wasm crossing sample `1399329970`, approximately 0.325806898, rather than the native sample above. This predates hourly pacing and is not a reason to weaken the native seed guard. Keep it explicit when comparing native campaign traces with browser reproduction; no RNG implementation change is proposed or performed by this audit.
