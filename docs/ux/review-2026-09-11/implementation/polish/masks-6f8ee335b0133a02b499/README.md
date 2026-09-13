# Masks protection — preview 6f8ee335b0133a02b499

The Masks equipment tag now halves the risk of a new illness. It does not erase an existing illness, skip its costs, or change time and distance. All UI and balance configuration were retained from Save / Load preview a2666947301cbcb4bfb7.

Validation: 192 game library tests passed, including deterministic protected/unprotected illness, non-immunity, existing illness, cooldown and save checks. Format and strict native/Wasm lint passed; release build completed. Both modes retained identical saves and the protection tag after offline reloads in isolated browser profiles; no browser errors. Preview publication verified all 119 files plus the service worker without accessing player storage.

Preview: http://127.0.0.1:8180/play/
Production remains 1c145d89fef4fbab75a9.

Experimental campaign work remains separate: applying this protection to the retained Clear ×3 tuning reduced individual pacing failures from 45 to 39 across 2,000 unchanged runs. A 14-day recovery-window trial reduced them to 34. Both trials still fail the two Balanced mean-distance guards and are not published. A focused rest diagnostic matched all 39 original results at the CSV's reported precision; it showed illness, low sanity and explicit encounter rest requests. No pinned tests, acceptance thresholds or tester strategies changed.
