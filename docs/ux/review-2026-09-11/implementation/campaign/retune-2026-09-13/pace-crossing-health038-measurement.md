# Crossing and health measurement — September 13, 2026

Measurement only. This configuration is not integrated, previewed or published. The existing guards, strategies, seeds, route scale, clock, activity costs and bribe parameters are unchanged.

Compared with pace-crossing, this changes only Balanced daily health decay from 0.12 to 0.38. Camp remains experimental +6. The source has 538 input files; the seed 4242 regression and release build pass. The original acceptance command exits 1.

| Metric | Classic Balanced | Deep Balanced |
| --- | ---: | ---: |
| Internal mean distance | 2104.940 | 2131.725 |
| Mean days | 21.052 | 21.644 |
| Boss reach | 40.40% | 64.80% |
| Boss wins | 23.20% | 20.80% |
| Early survival | 82.40% | 82.40% |
| Mean moving-day ratio | 95.54% | 94.56% |
| Bribe success | 74.11% | 74.11% |
| Terminal crossing rate | 10.81% | 10.81% |
| Dominant tracked failure family | 49.03% | 75.25% |

Classic boss reach, wins, survival and failure-family mix now meet their pinned bands. Average internal distance is still 4.94 miles above the allowed maximum in Classic and 31.72 above it in Deep. There are 125 individual movement-ratio failures: Classic Aggressive 40, Classic Balanced 9, Deep Aggressive 40 and Deep Balanced 36. The six remaining guard entries are retained in the analysis JSON.

All 1,500 non-Balanced result rows were compared field by field and remain identical to the parent campaign. The CSV footer contains a variable wall-clock duration, excluded from gameplay row comparison. Later validators were not reached after the acceptance command short-circuited on the first movement-ratio failure. This is not a full acceptance pass.

Binary SHA256: `3e41e7007d9197cc03a699363dcb045694a1b2ac67b37d1ffeee12a178cc0006`. The runner freshly touched Rust sources before compilation, verified the embedded canonical source path, and rechecked every input hash after the campaign.
