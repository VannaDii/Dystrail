# Gameplay retuning — September 13, 2026

This unit is ongoing. [Measured candidates and decisions](measurement-report.md) preserve the original experience thresholds, seed 1337–1586 matrix, and automated player strategies. Every full campaign has 2,000 rows; all still fail one or more retained guards. Compressed CSVs retain their original hashes in compressed-results.json. No fingerprint/experience assertion was changed.

The actual-driving pace-fatigue change passes eight new boundary/regression checks, 335 full native checks (with the same three retained config/replay failures), formatting and strict native/Wasm Clippy after four test initializers were refactored. It was verified in preview **4172ba426099ddcb6dcd** and remains included in the current **2b0a772d649e1b822dbd** poncho preview. Both preserve production camp +4 and exclude all experimental balance settings. Health-only and combined health040 trials remain unselected; camp +8 is rejected. [Preview and production delivery](../../polish/README.md) are tracked separately.

Subsequent isolated measurements preserve all 2,000 gameplay rows and the original CSV footer. The footer's variable wall time is excluded only when comparing gameplay rows, never from archived bytes.

| Candidate | Individual movement-ratio failures | Remaining aggregate failures | Selection |
| --- | ---: | --- | --- |
| Actual-driving pace fatigue | 207 | Balanced distance, boss reach/wins/survival and failure mix | Verified engine/UI unit in preview; campaign remains open |
| Pace + Balanced crossing weights 0.70/0.10/0.20 | 183 | Balanced distance and Classic boss reach/wins/failure mix | Measured only |
| Pace + crossing + Balanced daily health 0.38 | 125 | Classic Balanced distance 2104.9396; Deep Balanced 2131.7248 | Measured only; other numeric aggregates pass |
| Above + overnight activity settlement | 104 | Classic Balanced distance 2102.0052; Deep Balanced 2131.8124 | Measured only; three new Resource Manager movement failures require explanation |

[Health 0.38 measurement](pace-crossing-health038-measurement.md) and [activity archive](pace-crossing-health038-activity-archive.json) retain commands, exact sources, unchanged guards, CSVs and failed acceptance exits. The activity change has seven new regressions and 35 focused checks; it does not alter the explicitly lazy low-level clock contract. The separate checkpoint-arrival candidate is still being measured. No current candidate passes the complete experience contract.

The poncho fix has a retained failing-before regression: Storm ignored the store's rain_resist protection because its configured sanity mitigation was absent. The two-file correction protects Storm sanity only; supplies, distance, encounter chance and other weather remain unchanged. See [exact evidence](poncho-fix-evidence.json). Its separate preview unit now passes four actual equipped/unprotected Storm drives, 48 localized help checks and offline reloads; [preview proof](../../polish/ponchos-2b0a772d649e1b822dbd/README.md) retains the native and browser evidence.

The stale first pace release attempt is explicitly invalidated and retained. Subsequent candidate builds verify the embedded compiled-source path and all before/after input hashes, avoiding accidental Cargo reuse between isolated snapshots.
