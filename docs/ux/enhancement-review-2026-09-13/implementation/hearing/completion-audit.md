**Approved hearing: completion audit**

The approved implementation goal is complete for local release review. The user accepted 190–200 Classic/Balanced wins per 1,000 runs; the measured result is 197. Deployment is a separate step. The other enhancement ideas remain in the broader plan.

| Requirement | Implemented behavior and evidence | Result |
|---|---|---|
| Preserve preparation value | Existing `vote_chance` formula retained; entry snapshot freezes stats and base odds after existing policy preparation. Campaign arrival rate is unchanged. | Pass |
| One to three rounds | One mandatory round unless already exhausted; independent 50% continuation after each of the first two survived rounds; no fourth round. Scripted branch and distribution tests pass. | Pass |
| Influence and sanity | Uniform integers 50–150; 2 sanity per actual round; continuation and vote cost no sanity. Exhaustion prevents all subsequent draws and victory. | Pass |
| Average and closing outcome | Average actual rounds only. Automatic victory at raw adjusted odds ≥100% occurs only after questioning closes and sanity remains positive. Otherwise use the original integer draw 0–99 against the unrounded threshold. | Pass |
| Existing policy guarantee | Deep/Aggressive approval remains guaranteed after surviving questioning, including weak influence. Existing preparation costs are disclosed and charged once. | Pass |
| Suspense and player pacing | Arrival, preparation/rest, animated rounds, reading holds, automatic committee checks, closing calculation, vote and four distinct verdicts. Existing scenes and active crew are retained; chair/clerk reactions and concise satire support the beats. | Pass |
| Forecast and result explanation | Separate starting odds and survival forecast; current and prepared sanity explained; completed-round arithmetic and exact passing draw range recorded. A score target is not presented as a guaranteed vote. | Pass |
| Save and seeded replay | Versioned immutable report plus saved reveal cursor. Reload, Skip, Fast, repeated clicks and manual save/load preserve outcomes and charges. Old completed saves remain terminal. Questioning has a fixed independent stream; final voting keeps the old stream. | Pass |
| Clock and terminal behavior | Existing two-hour hearing time and day-ledger accounting retained; no post-verdict recovery or hazards. Tests cover crossing the day boundary. | Pass |
| Accessibility and layout | Reduced-motion path, keyboard/focus handling, dialog/hidden-page pause, readable result holds, desktop/phone and Arabic RTL coverage. Latest screenshots inspected. | Pass |
| Localization and offline | New text translated in English, Italian, Spanish and Arabic, with explicit English fallback in other bundles. Offline restore and new art covered; all 122 asset hashes and lengths verified. | Pass |
| Campaign acceptance | Exact CI campaign command passes across 8,000 runs. Classic/Balanced is 19.7%, within the user-accepted 190–200 range. Only its minimum changed from 20% to 19%; the prior ceiling and other limits remain. | Pass |
| Reviewable delivery | Local release revision `9f4c685e4e9207ccf7d7`, test checkpoint, playable preview, screenshots, validation logs, probability audit and asset provenance retained here. | Pass |

Core checks live in `dystrail-game/src/boss/hearing_tests.rs`, `dystrail-web/tests-e2e/hearing.spec.ts` and the existing state-screen tests. The accepted campaign boundary is covered in `dystrail-tester/src/logic/playability.rs`. Final counts and exact evidence paths are in [validation.json](validation.json).

The latest full browser run passed 24 cases and exposed a test expectation of `$20.00` where the existing formatter correctly displays `$20`. After correcting that expectation, both affected desktop/phone cases pass. Product currency formatting was unchanged; both logs are retained. All 26 final cases are covered on the current release revision.

The probability audit is reproducible with `python3 audit/calculate-probabilities.py`. Its input is the saved 1,000-run arrival export; `audit/arrival-probe.rs` records the temporary diagnostic used only in an isolated workspace copy. It did not change the shipped game or tester.
