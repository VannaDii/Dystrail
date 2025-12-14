# Daily Flow & Mechanics

This page walks through the exact order of operations for a single day, with the key formulas and thresholds used in engine code.

## Step-by-step day
1. **Player inputs**: choose pace (`PaceId`: Steady, Heated, Blitz) and diet (`DietId`).
2. **Travel miles**
   - Start from `travel.mpd_base` (family/overlay).
   - Multiply by pace factor (Classic: Steady 1.0, Heated 1.12, Blitz 1.3).
   - Multiply by weather factor (Classic: Clear 1.0, Storm 0.82, HeatWave 0.78, ColdSnap 0.9, Smoke 0.86).
   - Apply exec-order/weather/diet modifiers if configured.
   - Clamp to `[mpd_min, mpd_max]` (Classic: 10.0–22.0).
3. **Partial travel** (detour/repair/etc.): miles × `partial_ratio` (Classic 0.50; Balanced overlay 0.51).
4. **Wear accumulation**
   - Classic baseline: `wear = base (0.07) + fatigue_k (0.08) * fatigue(distance, comfort_miles=1250)`.
   - Balanced overlay: base 0.073, fatigue_k 0.245, comfort_miles 1750.
5. **Breakdown roll**
   - Classic base: `p = breakdown.base (0.01) * pace_factor * weather_factor + beta (0.10) * wear_component`.
   - Overlay tweaks: base 0.015, beta 0.185; pace multipliers Steady 0.94, Heated 1.08, Blitz 1.28; weather multipliers Clear 1.0, Storm 1.25, HeatWave 1.35, ColdSnap 1.1, Smoke 1.15.
   - Targeted part weights (Classic): tire 44, battery 22, alternator 18, pump 16.
6. **Crossing resolver** (if scheduled for the day)
   - Classic probs: pass 0.72, detour 0.16, terminal 0.12; detour costs 1–3 days.
   - Bribe: pass bonus 0.2, terminal penalty 0.2, diminishing returns 0.4 (Balanced overlay adjusts to 0.192 / 0.045 / 0.38).
   - Permit can disable terminal for eligible tags (default `["checkpoint"]`).
7. **Encounter resolver**
   - Uses cooldowns and history windows (`constants.rs`: cooldown 1 day, history window 10, repeat window 6, soft cap threshold 5).
8. **Daily tick (supplies/sanity/HP)**
   - For each channel: `base × pace × diet × weather × exec` (see `DailyTickConfig`).
   - Rounded to integers and clamped to stat caps. Classic family uses zeros; overlays add decay (e.g., Balanced `health.decay = 0.12`).
   - Rest requests can heal HP if `rest_heal` is set (overlays can enable).
9. **Endgame/boss gates**
   - Endgame (Deep) from `endgame.json`: starts at 1750 mi, guard at 1950 mi, health floors 45–50 HP, wear multipliers 0.6–0.7, stop caps (window 10, max 2 full stops), wear shave 0.7.
   - Boss gate: requires distance ≥ `distance_required` (defaults to 2100 mi). Boss chance is weighted by supplies, sanity, allies, pants penalty, and policy bias (`BalancedBossBias` in `boss.json`: Classic bonus 0.30, Deep multiplier 1.1, Deep bonus 0.08). Outcomes: PassedCloture, SurvivedFlood, PantsEmergency, Exhausted.

## Day record semantics
- `Travel`: full mileage credit.
- `Partial`: mileage multiplied by `partial_ratio` (detours, repairs, shared travel).
- `NonTravel`: camps, blockers, endgame/boss-only days.
- Travel ratio uses `(Travel + Partial) / total_days` and is checked against guards.
