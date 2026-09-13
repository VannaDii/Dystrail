# Crossing arrival

Driving is capped at the next crossing and resolves its event on arrival, preserving the exact route boundary rather than overshooting or requiring a second tiny leg. Crossing time/cost remains separate from driving, and a stop that fills the day closes the day correctly.

206 game library tests, format, strict native/Wasm lint and release passed. Native tests cover pass/detour/terminal outcomes, all crossing boundaries, round-trip conversion, time and RNG use. Browser checks cover both modes at morning and end-of-day arrivals: exactly 30 miles/minutes of driving to mile 1250, one crossing resolution, the correct additional stop time/day rollover, and no mutation after pausing or reloading offline. No browser errors.

Build 69bbf502bf4d6e27f55c retains the Load icon, masks and activity fixes; experimental campaign configuration remains excluded. An encounter-granted ride reaching a crossing still needs its separate caller-order review before that path can be considered fully verified.
