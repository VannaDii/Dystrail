# Completed continuity review

Reviewed the existing game design in installed Chrome and native Safari, with final evidence in [the West Coast bundle](../west-coast/README.md). Earlier findings below were fixed and verified. This review does not certify production readiness; the dependency and campaign gates are recorded separately.

| Flow | Finding and resolution | Evidence |
| --- | --- | --- |
| Start | Normal van retains all six travelers in paired rows. Character action aligns with the second mode and stacks on a phone. | Current Chrome onboarding suite |
| Character | Satirist preview previously showed 11 sanity even though play clamps it to 10. Preview now derives actual starting values. All six origins are on the West Coast. | character-origins.png; native preview helpers |
| Crew | Compact six-person grid, transparent busts, custom player/crew names and editable generated names retained. | Current onboarding/crew checks |
| Outfitting | Every starting item is an editable paid cart allocation; no hidden supply grant. Cash is whole dollars. | Current onboarding/store checks |
| Travel/map | Initial session distance differed from the first tick, shifting geographic progress. Session construction now applies the actual configured distance. Automatic maps retain queued decisions and resume in one click. | New route-scale regression; 60-case Chrome batch |
| Camp | Handle breakdown previously called a guarded travel action and did nothing. It now opens the existing part-specific choices without consuming a turn. Removed a stale close callback that could override outcome navigation. | Camp screenshots 04–10 in output/playwright/consistency; current tests |
| Conditions | Weather and policy impacts share the temporary-effect area. Optional tips remain separate from essential information. | Current Chrome/Safari checks |
| Town | Services, inventory and journal remain reachable. Benefits show the available amount before a visit and the actual claimed amount afterward. | Current Chrome/Safari checks |
| Local conversation | Replaced the article-like text block with a prominent resident remark, compact source record and aligned return action. Correctly stacks on a phone and mirrors in Arabic. | conversation-en-1280.png, conversation-en-390.png, conversation-ar-390.png, Safari conversation evidence |
| Outcomes/journal | One Outcome heading, typed actual cash/parts/condition deltas, no Pants stat, persisted ally notices, clear dated journal cards. Raw crossing-result keys are now localized. | Current Rust/Chrome/Safari checks |
| Ending | Final scene and primary traveler match the actual cause and route location; complete scorecard is visible. | Current onboarding-results suite |
| Offline | Complete build is cached before play; new complete updates precede launch and failed updates retain the last complete version. | Current Chrome preload/PWA tests; all 88 HTTP hashes |

Every new western road background and representative western town composition was opened and inspected in its rendered game scene. These are regional illustrations; exact road geometry and city data come from the sourced map/profile records. No unique street-scene claim is made for all 51 towns.
