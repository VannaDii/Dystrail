# Font unit verification

The bundled fonts and explicit launch preparation are verified. The original font candidate was d1ca5ad6aa0b5c95ec14; the final recovery correction is in candidate 4831b874f6a8ad955112.

- All seven faces were loaded when the launch gate was removed, with all 67 images decoded.
- All seven cached WOFF2 file lengths and SHA-256 hashes matched the frozen manifest.
- Previously unused normal/italic/bold/Arabic probes caused zero font requests after launch.
- A completely closed/reopened disposable Chrome profile launched offline with all font hashes correct.
- Chrome platform-font glyph inspection selected actual Atkinson Regular/SemiBold/Bold/ExtraBold instances at400/600/700/800, the actual italic face, Courier Prime Bold, and corresponding real Noto Arabic weight instances. These are custom bundled fonts, not system fallback or a single static weight.
- Side-by-side desktop/phone specimens were visually inspected. Atkinson's slashed zero appears at400 as well as heavier weights. Its800 weight is visibly strong at small UI sizes;600/700 are clearer for small HUD labels/numerals. No font decoding or variation-axis defect was found.
- Missing font bytes with a deliberately unavailable worker fetch retained the gate and retry control; reconnecting and Retry restored launch.
- Same-length corrupt font bytes retained the gate. The initial candidate could not repair the entry; the final loader deletes that invalid font and the current readiness marker, allowing Retry to fetch the exact verified bytes. Final regression passes with all seven faces loaded and a correct current marker.

Only offline-client.js changed for the final recovery fix. Its frozen SHA-256 is 67b59d76d1abe2c3bf02f989c5e7a40e14ed4142ccddb4292e5caf08e4d401ed. All font assets/CSS remain unchanged from the validated font bundle.

Missing-network faults were explicitly simulated in the disposable worker because Chrome's page offline emulation did not block worker-initiated fetches in this configuration. This distinction is recorded in the reports; no shared profile or server/source code was changed. The first missing-font harness timeout and the real initial corrupt-retry failure remain preserved.

Evidence: unit2/report.json, unit2/font-weights-desktop.png, unit2/font-weights-phone.png, unit2/fault-report.json, unit2b/fault-report.json, unit2b/missing-font-gate-phone.png, unit2b/corrupt-font-gate-phone.png. The final fault runner exited0. No Rust builds or repository test files were changed by this task.
