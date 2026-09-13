# Combined timing and poncho diagnostic

This is an isolated measurement candidate, not acceptance of the Activity policy or a preview/publication artifact.

Base: final tested checkpoint-arrival candidate. Four files differ from that base:

- `dystrail-game/src/activities.rs`: the exact already-reviewed seven-line overnight settlement and seven focused tests, copied byte-for-byte from the measured Activity candidate.
- `dystrail-web/static/assets/data/weather.json` and `dystrail-game/tests/weather_paths.rs`: the exact two-file poncho fix and its regression, copied byte-for-byte from root's checked poncho source.
- `dystrail-game/src/travel_time.rs`: exactly four test initializers from the checked root file. All runtime text before the test module and every assertion remain byte-identical; an explicit four-replacement transformation verified the entire file.

All other 535 files match the final arrival candidate. Balanced health decay remains 0.38, crossing pass/detour/terminal weights remain 0.70/0.10/0.20, and camp sanity remains 6. All tester strategies and acceptance guards are byte-identical. No shared product source, existing browser origin, preview or production is changed.

Component provenance, full before/after file hashes and the complete 539-file input set are in the companion inputs JSON. Focused existing/new tests, strict Clippy, seed 4242, immutable tester build and the unchanged 2,000-run campaign remain required. Cargo must be released after copying and canonical-path verification of the immutable tester, before its campaign run.
