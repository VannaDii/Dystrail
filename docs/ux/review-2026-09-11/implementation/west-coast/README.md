# West Coast routes and local conversations

Current preview: [Play on port 8180](http://127.0.0.1:8180/play/). Build `f6b4922d52c336b3eec9`. Start a new run to use the new origins. The user explicitly waived compatibility with prior runs; no saved-route migration layer is retained.

| Character | Starts in | Road distance | Route locations |
| --- | --- | ---: | ---: |
| Journalist | Seattle | 2,921 mi | 18 |
| Organizer | Portland | 3,053 mi | 17 |
| Whistleblower | San Francisco | 3,020 mi | 17 |
| Lobbyist | Los Angeles | 3,037 mi | 19 |
| Staffer | Sacramento | 2,956 mi | 18 |
| Satirist | San Diego | 3,370 mi | 23 |

Each route crosses at least five geographic regions before D.C. The routing data contains actual road geometry and cumulative road distance; region, weather pool, available encounters, scenery, towns, and the map all follow the same route position. The simulation's configured distance remains independent from physical road miles. It is now applied when the session is created, so the first travel day cannot change the map scale beneath the player.

## What changed

- Added 23 western towns, bringing the inventory to 51 exact locations with sourced 2020 city populations, attractions, dated administration-impact records and fictional resident commentary.
- Added Pacific Coast, Mountain West and Southwest weather and forage profiles, five matching road backgrounds and four western town/two western camp settings. The normal van still contains three paired rows of independent active occupants.
- Added six western encounters and eighteen choices. Together with the expanded regional eligibility of existing national incidents, the inventory contains 65 encounters and 187 options. These use 12 shared encounter compositions; this is not 65 bespoke illustrations.
- Rebuilt Word from the town as a resident's remark and a compact local-record card, with a direct source link and contextual explanation. Desktop uses aligned columns; phone widths stack them. Source facts and fictional satire remain distinct. The town scene, player/NPC cast, inventory controls, repeat access and claimed reward remain intact.
- Removed stale origin names from map captions. Captions interpolate the actual route origin and are localized in English, Spanish, Italian and Arabic.
- Fixed the starting-stat preview, the camp breakdown action, camp outcome routing and untranslated crossing-result keys found during the continuity pass.

## Evidence and limits

[Validation record](validation.json), [validation workflows](commands.md), [regional scene checks](regional-scenes.json), [conversation language checks](conversation-localization.json), [Safari checks and measured viewports](safari-validation.json), [art briefs](art-briefs.md), [source inventory](../scene-flow/town-conversations.md).

247 Rust tests, strict Clippy, formatting and release builds pass. Engine coverage is 79.05% against the required 77%. The full selected Chrome batch passed 60 checks on the route/layout build; the final caption/log refinement passed 24 additional targeted checks. Native Safari 26.6.2 passed 11 checks on the final build. Safari ignored window-resize requests and stayed at 1728 CSS pixels, so those are desktop results; phone layouts and Arabic/RTL were verified in installed Chrome. No physical phone installation was performed.

The complete offline bundle contains 88 files and 90,547,018 bytes. Every file served at 8180 matched its manifest size and SHA-256. Chrome verified full offline relaunch, update-before-play, asset repair and failed-update fallback. Safari's earlier broader offline run is retained in the preceding review; the current Safari pass covers the revised gameplay and conversation flow.

Dependency audit and campaign acceptance remain separate release gates; see the current validation record. No production deployment, branch, commit or PR was made.
