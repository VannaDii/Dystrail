# Cast and road image decisions — approval draft

This is one completed inspection batch within the full inventory, not approval to produce or a claim that the feature is complete. No image was generated or edited. Original files remain byte-for-byte preserved.

## Six cast sheets

Propose reusing these six originals. Each is **1774 × 887**, with a nominal **4 × 2** pose grid (**443.5 × 443.5** native-coordinate cells). The recovered renderer uses the equivalent 2048 × 1024 logical coordinate system; that is a uniform scale, not an aspect-ratio change. Its portrait inset still needs a rendered edge check.

| Role | Durable original | Decision |
|---|---|---|
| Journalist | [exec-e440f529-aa7b-4569-9619-2407985c6da2.png](../../../../recovery-2026-09-21/generated-originals/exec-e440f529-aa7b-4569-9619-2407985c6da2.png) | Reuse proposed; zero generation |
| Organizer | [exec-da782290-9aa5-40d5-92fc-bb2f6fcebe28.png](../../../../recovery-2026-09-21/generated-originals/exec-da782290-9aa5-40d5-92fc-bb2f6fcebe28.png) | Reuse proposed; zero generation |
| Lobbyist | [exec-10125800-62e6-441f-b4bf-ea30e41b6c95.png](../../../../recovery-2026-09-21/generated-originals/exec-10125800-62e6-441f-b4bf-ea30e41b6c95.png) | Reuse proposed; zero generation |
| Satirist | [exec-1d76e05d-3e0d-40e9-a42e-8acd86646638.png](../../../../recovery-2026-09-21/generated-originals/exec-1d76e05d-3e0d-40e9-a42e-8acd86646638.png) | Reuse proposed; zero generation |
| Staffer | [exec-294e7ae1-5c1c-4d85-b956-f5828aeeef84.png](../../../../recovery-2026-09-21/generated-originals/exec-294e7ae1-5c1c-4d85-b956-f5828aeeef84.png) | Reuse proposed; zero generation |
| Whistleblower | [exec-3cac2b7c-f572-4890-b980-b718cdf93e88.png](../../../../recovery-2026-09-21/generated-originals/exec-3cac2b7c-f572-4890-b980-b718cdf93e88.png) | Reuse proposed; zero generation |

The inspected sheets supply standard/happy/defeated/unwell portraits plus standing, near-passenger, far-passenger, and driver poses. Preserve their identities, range of skin tones, hair, age presentation, and feminine/neutral styling. No religious symbols were observed. Do not infer a character biography or ethnicity from appearance.

**Composition and placement:** keep the original frontal/three-quarter portraits in existing character-selection, naming, and result slots. In world scenes use the standing/seated poses, never an extra caption-side bust. Use actual party membership and status. Keep an empty seat when the corresponding traveler is absent unless existing party rules explicitly reassign it. Respect the current driver assignment. A seated pose must align with its own seat back and be clipped by its own window/door geometry. The two passengers in a row must have separate head and shoulder positions; no lap-like arrangement or intersecting torsos.

**Perspective and proportions:** preserve the cast’s stylized anatomy. Use one common person scale at a given depth, a consistent ground plane, and the physical anchors in README. Do not independently enlarge heads or shrink the van to make a scene fit. The far passenger can be partially occluded by the near passenger, but must not be child-sized. Driver hands align with the wheel. Seat belts stay on the corresponding torso.

**Desktop/mobile:** one atlas and one scene coordinate system. Scale the assembled scene uniformly. At 1440px desktop and 390px mobile review widths, check six, three, and one surviving traveler, each possible driver, and empty parked van. These are acceptance cases, not a new interface design. Keep captions over the bottom-left of the actual image. Preserve the approved stats component.

## Road and vehicle originals

| Asset | Exact original dimensions | Decision and composition |
|---|---|---|
| `journey/open-pacific-northwest.png` | 1536 × 1024 | Reuse proposed. Empty horizontal road, mountain/lake setting, no baked crew. Side-on vehicle layer follows the road. |
| `journey/open-mountain-west.png` | 1536 × 1024 | Reuse proposed. Empty horizontal road and mountain/grassland setting. Same road camera and vehicle scale. |
| Corrected Appalachian original `exec-9230d055-c943-44d4-bc45-53da1100bcc3.png` | 1536 × 1024 | Reuse proposed. Retain the repaired skyline, geography and road. No religious symbol observed in this inspection. |
| `journey/van-empty.png` | 1536 × 1024 | Reuse body with surviving mask/composition, pending rendered edge check. Raw image contains a checkerboard; it is not a ready transparent sprite. Existing `journey.css` applies `van-crew.png` as a mask. Preserve/recover masking rather than generating another van. |
| `journey/road-classic.png` | 1536 × 1024 | Exclude from new world-scene binding: fixed old crew and visible steeple. Propose an existing crew-free regional road plus actual crew and editable sign layers instead. No replacement generation requested. |
| `journey/road-deep.png` | 1536 × 1024 | Exclude from new world-scene binding: fixed old crew and baked sunset/billboard. Propose the same approved road/crew layers with mode presentation and localized signage. No replacement generation requested. |
| Foundry correction `exec-cdd7f444-9720-419d-a646-1a479613eb69.png` | 1536 × 1024 | Hold for user review; apparent church-like silhouette remains at far left. No retry authorized. A suitable existing region background may be used only if geographically/source appropriate. |
| Wide crown checkpoint `exec-69ac49e0-724b-49f6-85c8-bb41cc09ffe5.png` | 1536 × 1024 | Reuse candidate, not accepted composite. Small clerk behind counter; camera sees the whole booth and clear foreground road. Exact source/outcome and van scale still need verification. |

For each reusable road, preserve the full 3:2 source and shared world coordinates on desktop/mobile. The full road, vehicle and required story subject must remain visible; do not fix a small screen by enlarging an NPC, shrinking the van, or independently cropping layers. Caption reserve stays at bottom-left with the existing overlay treatment. Any source-specific scene that cannot fit those constraints returns to the inventory as an unresolved composition, not an automatic image request.

## Presentation work that needs no generated image

- **Billboards:** recover the existing localized HTML/CSS sign implementation. Blank physical sign plus editable localized copy; deterministic selection from saved seed/day/region without consuming game RNG. No painted lettering or per-language image generations.
- **Time/weather:** recover clock-based lighting and weather layers over the same scene. Preserve actual simulation hour; menu time does not advance it. Day ending is not automatically sunset. Indoor scenes should not receive outdoor precipitation. Preserve pixel-art readability and contrast.
- **Captions/stats:** recover bottom-left scene overlays and remove the supplementary bust. Use existing approved stat designs; no invented result layout.

## Remaining inventory work

Bind the remaining source-specific scenes/props to retained originals and atlas cells; inspect those selected for use. The 644 source records remain accounted for separately, with 36 mechanics-dependent records inactive. This batch does not convert source counts into art coverage. Do not ask for production approval until the complete necessary-image list and unresolved exceptions are reviewable.

## Additional shared-background inspection

- **journey-settings-v1.png — partial_reuse_only:** 1536×1024, 2 columns × 3 rows. Exclude cell 0 (red cross on booth) and cell 3 (white cross on red kit). Cells 1 and 2 are regional context candidates. Cell 4 contains baked sunset and ballot-box staging, unsuitable for arbitrary arrival time. Cell 5 is a real rest-area candidate only; not a universal failed-ending location.
- **western-settings-v1.png — reuse_region_matched_candidate:** 1536×1024, 2×3. Cells 0–3: Pacific Northwest, California, mountain, desert towns; 4–5: desert and mountain camp. No religious symbols observed in inspection. Confirm divider pixels before cropping; current equal-third logical rows do not prove exact painted gutter boundaries.
- **open-heartland-prairie.png — hold_symbol_ambiguity:** 1536×1024. Tiny white background tower may read as a steeple. Leave held for user review; orchard is an existing regional alternative.
- **open-heartland-orchard.png — reuse_candidate:** 1536×1024. Horizontal road, orchards, lake and low bridge. No religious symbols observed. Regional context only.
- **open-rustbelt-lakeside.png — reuse_candidate:** 1536×1024. Horizontal road and industrial waterfront; no religious symbols observed. Existing alternative to held foundry where route geography fits.
- **open-beltway-suburbs.png — reuse_candidate:** 1536×1024. Horizontal road, houses and offices; no religious symbols observed.
- **open-great-basin.png — reuse_candidate:** 1536×1024. Horizontal road and open basin; no religious symbols observed. Use only for matching route geography.
- **camp.png — exclude_fixed_cast_and_time:** 1536×1024. Baked old journalist/organizer, van, orange sunset and stars conflict with actual surviving cast and clock. Do not restore as the general camp scene.

All decisions remain inventory proposals. Reusing an eligible background does not complete a source gag or authorize image production.

## Remaining town context and supporting-cast constraints

- **encounter-settings-v3.png:** 1536×1024, 3×2 grid of 512×512 cells: motel desk, cafe, library, museum, farm office, public-service counter. No obvious religious symbols in direct inspection. Existing perspective/room props can be reused only when the source setting fits. Square cells cannot be stretched into 3:2 scenes; contain at native aspect or approve another composition. Fine background texture needs comparison with the final cast at actual display size.
- **town-npcs-v1.png:** 1536×1024, 3×2 grid of 512×512 portraits. Cells 1 and 2 have masculine presentation inconsistent with the requested cast direction; exclude those designs. Other cells are reuse candidates only where a portrait is an approved existing interface element. No supplementary caption busts; no portrait-as-full-body substitution.
- **Corrected supporting-cast original `exec-dddd7915-51c6-4bf4-a93e-b2b1e680a9b6.png`:** six 512×512 portrait cells on a 1536×1024 canvas. Hold for user review: no obvious religious symbols, but painterly detail and portrait-only poses do not satisfy the in-world cast requirement. No automatic replacement generation.
