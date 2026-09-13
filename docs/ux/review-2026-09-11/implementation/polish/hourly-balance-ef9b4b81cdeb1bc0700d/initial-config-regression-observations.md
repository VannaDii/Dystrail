# Initial integrated unit-test observations

The original first-run log path was reused by the final build. These observed failures are retained from the actual tool output; this file is a transcription, not the original raw log.

- Overnight gathering: actual sanity 7, old expectation 6.
- Overnight work: actual sanity 5, old expectation 4.
- Quiet daily recovery: actual sanity 6, old expectation 5.
- Rest after work: actual sanity 10, old unbounded expectation 13.

The first three reflect the accepted +1 increases to Quiet/Mixed daily recovery. The fourth verifies the unchanged sanity cap of10. Exact regression expectations were updated; production clock behavior was not changed to address these failures. Final native tests pass. The separate full-suite pre-fingerprint log is retained as workspace-complete.log.
