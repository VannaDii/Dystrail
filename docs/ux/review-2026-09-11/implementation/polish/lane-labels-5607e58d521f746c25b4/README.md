Preview **5607e58d521f746c25b4** is published at http://127.0.0.1:8180/play/. All 119 assets (109,967,489 bytes) and the service worker match the frozen artifact. Saved games were untouched; production remains 1c145d89fef4fbab75a9.

The driving van now follows the road artwork's lower-edge geometry. Parked van/crew coordinates are unchanged. Italian, Spanish and Arabic speed labels are translated and keep the speed and unit together. Camp remains +4; all gameplay data and narrative copy are unchanged.

Validation: 96 tire-clearance comparisons across both modes, 12 road images and 320/393/1440/1920 widths; minimum clearance **11.65px** at maximum +2px suspension. Eight parked-layout comparisons matched exactly. The final build passed 18 localized label layouts at 320/393/1440, offline saved-state reload, and zero browser errors. Build exited 0; all asset hashes and /play/ base were checked before navigation.

The first d9 label check found stranded units at 320px; the three nonbreaking-space corrections resolved those failures. Its mocked-clock geometry run emitted two timer/closure errors on resume; that raw result is retained in browser-results.json. Final normal-clock browser checks had no errors. CSS and artwork are byte-identical between that geometry artifact and the published correction.

[Desktop scene](lane-classic-desktop.png) · [Phone scene](lane-deep-phone.png) · [Italian labels at 320px](labels-classic-it-320.png). Screenshots were visually inspected.

Frozen source: /tmp/dystrail-lane-label-preview/label-source. Artifact: /tmp/dystrail-lane-label-preview/label-web. The initial publication swapped dist successfully but HTTP verification met an orphaned preview server with empty responses. Restarting the same localhost server with retained logs restored service; all HTTP hashes then passed. No production deployment or browser storage changes occurred.
