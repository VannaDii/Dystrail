# Repair scenes

September 15, 2026. Repair artwork and runtime integration are reviewed for all four existing failures and their A/B/C source variants. This is one completed surface within the larger, incomplete feature.

## Runtime and staging

`journey_scene::repair_art` consumes the saved unit, actual part, shop/roadside location and committed choice. Offers show the failure; successful choices show the fitted replacement. The existing spare, cash, supplies and work-for-repair actions retain their costs, duration, availability and simulation randomness. No new repair mechanic is introduced.

The actor is selected from currently present crew. All six established feminine/neutral, racially diverse identities have an inspection pose; departed crew do not reappear. No supplemental UI bust is rendered. The route and title remain over the scene's bottom-left.

| Family | A | B | C |
| --- | --- | --- | --- |
| Alternator | Dark phone and radio beside the failed alternator | Small phone held near the open hood | Radio held; unused wrench nearby |
| Battery | Blank notepad and pencil | Dark dashboard inside the van; radio and lit phone nearby | Radio held; two battery terminals visible in detail |
| Fuel pump | Pump illustration and plain calendar nearby | Radio beside the stopped van | Call-icon phone held; radio and tools nearby |
| Tire | Lunar-drill tablet held beside the collapsed tire | Blank quote held; phone/radio at the van sill | Radio, used coffee pod and blank leaflet nearby |

The roadside uses actual route scenery and existing clock/weather treatment. The workshop uses a new windowless pixel-art background with a clear floor. Outdoor weather does not enter the shop. Close-up panels distinguish the actual failed and fitted components without implying spare ownership before a choice.

## Asset handling

Seven immutable raster sources supply the workshop, six inspection poses, components, van, props and two matching silhouette masks. Built-in image generation produced all raster artwork. Native SVG masks and measured atlas crops remove baked checkerboards without editing the source pixels. Uneven van-cell padding, neighbor bleed, prop alignment and dashboard persistence were corrected through rendered review. No religious symbols were observed in the inspected sources or compositions.

Source identity and hashes: `review/art-satire/repair-final-source-integrity.json`. Earlier rejected transparent-output attempts remain recorded in `repair-source-assets.json`; the original opaque van/props are now usable only with their matching masks.

## Validation and limits

See `review/art-satire/repair-complete-review.json` for exact candidate and evidence. Native tests verify selection and existing repair behavior. Desktop/mobile browser cases exercise twelve variants and four actions, surviving/absent crew, indoor storms, English/Spanish/Italian/Arabic copy, cashless recovery and unchanged saved outcomes after reload. Actual scene screenshots and supplied-game-client captures were inspected.

Repair narrative/outcome copy is complete in English, Spanish, Italian and Arabic. Sixteen other locales still use the explicit English fallback; this remains part of whole-feature localization work. A prior departure receipt also retains English fallback in the Arabic fixture and is not counted as repaired by this work.

Remaining feature work includes care staging, other mapped narrative families, supporting-cast/localization coverage, dependent imagery, initial package size and combined route/mode/accessibility/save/replay/offline/hearing acceptance. The 36 mechanics-dependent records remain inactive. No deployment.
