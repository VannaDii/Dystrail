# Mileage ledger — 952fb2acea8c52d8500b

A full-precision campaign probe found actual final positions of exactly 2,100 miles but mean ledger mileage of 2100.000447265625. Nominal hourly increments and float accumulation diverged. The fix preserves sub-odometer distance in two defaulted ledger remainder fields across legs/days/saves, derives each day's distance from the precise day endpoints, and aggregates day history at higher precision. The existing daylight-distance assertion and <=2,100 campaign guard are unchanged. No tuning/config/content changes are included.

Two focused regressions cover fractional leg precision and multi-day distance conservation through a saved unfinished final day. An initial approach that credited only rounded odometer changes broke the existing daylight-distance assertion; it was rejected, with evidence retained. The final implementation passes all 213 engine and79frontend tests, strict native/Wasm lint, formatting and release build.

Browser verification passes both modes at1440/393width, six real hourly actions each (24hours total), with fractional controlled speed, open/completed records and exact saved-state continuation through offline reload after hours2/5. No runtime errors or horizontal overflow. The inspected phone capture retains the established presentation. Initial browser harness corrections accounted for flattened save fields and initialized the record required by its already-started-day fixture; these did not alter production code.

The separate retuned campaign trial now passes the mileage guard and has one remaining moving-day-ratio failure out of2,000runs. It is not included in this preview.
