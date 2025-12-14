# Balance Targets & Acceptance Guards

These are the numeric rails that keep runs inside the intended “Oregon Trail” feel and the gates the tester enforces.

## Global pacing targets
- Route length target: ~2,000–2,400 mi (boss gate at `distance_required`, default 2,100; overlays may raise victory miles).
- Day window: 84–180 days.
- Mean miles per day: 10–20.
- Travel ratio: ≥ 0.90 (guards block configs that drop below this).

## Classic family expectations
- Boss reach: 30–50% of runs.
- Boss win: ~20–35%.
- Survival (any non-early wipe ending): 60–80%.
- Failure mix: no single family (vehicle, sanity, exposure, crossings) > 50% of failures over large samples.
- Aggressive: lower survival and more terminal crossings than Balanced.
- Conservative / ResourceManager: higher survival and boss reach but boss win should stay ≲ 40%.

## Deep family expectations
- Same distance/duration/mpd bands as Classic.
- Higher per-run variance is allowed; means must still satisfy the Classic bands.
- Crossings/failures may be a bit harsher or stranger, but not arcade-short or ultra-long.

## Guard rails (from journey config)
- `guards.min_travel_ratio`: Classic 0.90; overlays may adjust.
- `guards.target_distance`: Classic 2000.0; overlay can raise (e.g., 2400.0).
- `guards.target_days_min/max`: 84 / 180.
- Tester sweeps (`dystrail-tester`) validate these across many seeds; failing guards fails CI.

## Crossing & breakdown reference numbers
- Classic crossings: pass 0.72, detour 0.16, terminal 0.12; detour days 1–3.
- Bribe: pass bonus 0.2, terminal penalty 0.2, diminishing returns 0.4 (Balanced overlay tweaks).
- Breakdown (Classic base): `p = base 0.01 * pace_factor * weather_factor + beta 0.10 * wear_component`.
- Overlay example (Balanced): base 0.015, beta 0.185; pace multipliers 0.94 / 1.08 / 1.28; weather multipliers 1.0 / 1.25 / 1.35 / 1.1 / 1.15.
