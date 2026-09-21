# Image production specification — review draft

September 21, 2026. Planning only. No game files changed and no images generated in this resumption.

## Production limit

- Reuse suitable existing images unchanged. A narrative variant, locale, outcome, time of day, or viewport does not by itself justify another image.
- Generate only an individually specified, necessary image. At most one attempt for that image; zero attempts when existing material suffices.
- No automatic correction passes, retries, alternate versions, or renamed replacements for a failed attempt. Preserve it for the user's review and continue independent work.
- Do not generate from this draft. First finish the asset decisions and review the specification with the user. Work remains single-agent.

## Current evidence and recovery issue

The registered feature worktree `/private/tmp/dystrail-art-satire` is absent. Git reports its registration as prunable. Its branch still points to `c56bc90`; this does not preserve the uncommitted integration work. The main checkout remains at `f1ed4fe` with Hearing changes and other existing modifications; those were not changed by this inventory.

There are **176 retained generated PNGs** in the thread's generated-images directory. [The inventory](retained-images.csv) records every original path, exact source dimensions, SHA-256, candidate filename mapping, disposition, and generation count. It neither transforms nor copies the art. Recorded history now yields filename information for 123 images, including 89 with explicit structured mapping evidence; 53 lack a recovered filename mapping. All 176 original generation briefs have been recovered. Some filenames have multiple historical versions. A recovered filename is not evidence of acceptance or current use.

Open the [browsable inventory](image-inventory.html) for every retained original, its exact dimensions, hash, original composition brief, filename evidence, and current review hold. Images are unchanged and loaded only for inspection.

All 176 are held for reuse/review, with **zero new generation requests**. This is an inventory of retained files, not a completed list of necessary production assets. Recovery has extracted 262 historical file candidates, including briefs and source metadata, without executing the old commands. Of the recorded file events, 166 could not be applied with an exact known base/context. These are not restored current source. The missing final integration catalog and unresolved scene bindings still prevent a complete necessary-production list. Do not replace the missing worktree by regenerating its images.

The surviving content checklist contains 644 editorial records. It is not a request for 644 illustrations. The 36 records depending on unapproved encounter mechanics remain outside active image production.

## Required fields for each necessary image

No image enters production without a row containing:

1. Stable asset ID and exact scene/state IDs it serves; the essential visual action or joke.
2. Existing original/reference and a concrete reason reuse cannot satisfy the scene.
3. Disposition: **reuse unchanged**, **layout/code correction**, **user review**, or **one generation needed**.
4. Exact output width and height; aspect ratio; transparency; any atlas cell dimensions and safe gutters. Retain suitable originals at their current dimensions.
5. Camera type, horizon/vanishing point if applicable, ground-contact line, depth planes, and bounding boxes for all principal figures, vehicle, props, and caption exclusion area.
6. Cast identities and allowed poses; which actors must be runtime layers because they can be absent.
7. Desktop and phone fit/crop rules; the focal action must survive both. No separate phone painting unless the existing image demonstrably cannot serve both.
8. Forbidden elements, state-specific differences, localized text surfaces, source reference, and maximum one attempt. Record the result without silently changing the brief.

## Hard visual rules

These rules implement the user's existing constraints. Numerical staging proposals in the next section still need review.

- **Pixel art throughout.** Deliberate visible pixel clusters, consistent edge density, limited stepped shading. No photographic textures, soft painterly detail, depth-of-field blur, or realistic rendering mixed with pixel characters.
- **No religious symbols.** No crosses, devotional emblems, religious jewelry, religious insignia, or distinctly devotional buildings. Satirical prose references are separate from image content. Preserve required US route geography; do not add religious shorthand for any region.
- **One camera per composite.** A side-view van cannot be pasted into a conflicting overhead or steep three-quarter ground plane. Every inserted figure and prop must match the scene camera and depth.
- **One scale per depth plane.** Use human height and van body dimensions as anchors. Characters farther behind a foreground van must not appear larger solely to make the joke readable. Camera changes require restaging the whole composition.
- **Physical contact is explicit.** Feet, tires, booth foundations, chairs, and barriers rest on their assigned surfaces. Seated bodies fit their seats; hands meet the intended props. A barrier spans the lane and clears the van when open.
- **Proportions cannot be a layout workaround.** Do not shrink the van to expose a booth or enlarge an official to advertise a joke. Solve obstruction through composition, camera framing, or a separately planned detail view.
- **Crew continuity.** Feminine or gender-neutral, racially diverse cast with consistent identities. Use actual surviving travelers only. Keep crew layers separate where membership changes. No overlapping seated bodies suggesting one person is sitting on another.
- **Text remains editable.** Billboards, labels, forms, and announcements use localized live text over blank artwork surfaces. No generated lettering, baked statistics, or invented interface designs.
- **Caption stays over the lower-left scene.** No detached character bust. Reserve quiet space without placing essential action or vehicle occupants behind the caption. Match the approved UI/stat components.
- **Time and weather are shared presentation layers where sufficient.** Reuse scenes for daylight, dusk, night, rain, smoke, and cold; do not order a new painting for each condition. Preserve legibility and avoid tinting interface text.
- **Responsive layout preserves geometry.** Scale a coherent composition together. Do not independently alter the van, people, or booth at phone breakpoints. Do not mirror editable lettering or a vehicle whose handedness matters.
- **One review, no image rescue loop.** Inspect the generated result once against its brief, record defects, and leave failed output for the user. Layout bugs are not permission to generate another image.

## Proposed measurable staging baseline

This is a proposed reference for new or genuinely replacement roadside scenes, not permission to resize existing accepted art or redesign the UI.

| Item | Proposed requirement |
| --- | --- |
| Single roadside/checkpoint scene master | 1536 × 1024 px, 3:2; no atlas unless individually specified |
| Pixel construction | Coherent 384 × 256 logical grid at 4× master size; match references before accepting |
| Camera | Level side view for the road and van; verticals upright; no wide-angle distortion |
| Human/van anchor | Representative adult 1.70 m; van body 4.80 m long × 2.10 m tall, excluding luggage |
| Same-depth size ratios | Adult height about 35% of van body length and 81% of body height; varied adult heights 1.55–1.90 m; within 10% of the declared scene projection |
| Architecture anchors | Counter 0.9–1.1 m; door 2.0–2.2 m; booth interior tall enough for its actual occupant; no giant clerk in a tiny building |
| Body construction | Preserve the approved cast’s stylized body/head proportions; do not impose realistic anatomy or alter identity. Occupants’ heads fit within windows at the same vehicle scale |
| Scene planning coordinates | Every principal object gets an explicit rectangle and contact point in the 1536 × 1024 master; depth offsets documented before generation |
| Caption reserve | Initial planning reserve x=48–1056, y=820–992, kept quiet; confirm against actual localized caption height before approving the image brief |
| Desktop/phone | Same image and camera. Validate 1440 px desktop and 390/320 px phone layouts, plus long/RTL and enlarged text; fit the scene before considering crops |

Do not use a single forced horizon or van position across interiors, close-ups, portraits, and outdoor scenes. Those need their own declared camera and dimensions. Never apply the physical size ratios to a headshot; they apply to people and vehicles sharing a world scene.

## Reuse decisions by game surface

| Surface | Default decision | What could justify a new image |
| --- | --- | --- |
| Six main characters and existing expression sheets | Recover selected originals; preserve identity and atlas geometry | A specific missing required pose or a user-rejected identity after the retained alternatives are checked |
| Van and seating | Reuse vehicle; correct seat positions, masks, and surviving roster in code | Vehicle art itself cannot support physically separate seats; specify the exact missing asset first |
| Road, stopped-road, regional scenery | Reuse route scenery with shared time/weather layers | A required region lacks suitable art, not merely a different hour or outcome |
| Billboards | Reuse/create simple native sign geometry and live localized copy | A necessary illustrated joke cannot be expressed with existing art or sign treatment |
| Encounter/town/activity satire | Reuse the correct setting and props across compatible variants | The narrative requires a genuinely different visible action/setting; bind it to exact source units |
| Crossing passage/diversion/failure | Reuse setting; preserve vehicle scale and animate/move actual layers | A necessary prop/pose is missing; a new entire image for each outcome is not the default |
| Policy announcements | Reuse retained bulletin art and approved presentation | A specific order lacks a suitable illustration after existing sheets are reconciled |
| Care, repairs, ally departures | Reuse settings and state-aware cast/props | A distinct necessary action cannot be represented faithfully by those layers |
| Hearing and endings | Preserve Hearing work, chamber, expression selection, and actual report/state | A confirmed missing visual beat; copy variants alone do not justify separate endings art |
| Logos, item icons, utility/reference records | Keep suitable existing assets | An identified defect within this feature's scope |

## Known last-batch crossing originals

All nine are retained at **1536 × 1024**. All are **user-review/reuse candidates**, not approved replacements and not requests to generate again. The original filenames and hashes are in the retained inventory; UUIDs identify those originals even while the worktree is absent.

| Intended asset | Source UUID | Required composition/meaning |
| --- | --- | --- |
| crossing-map-v2 | e463d6a5-198d-4845-8d09-ad0cac4babd7 | Official holds the map visibly at arm's length; map stays a hand-sized prop, with no oversized official |
| crossing-map-folded-v2 | 459b161a-18eb-4e55-a8c7-ea0a57bedc54 | Same official/camera/booth; folded-map passage state |
| crossing-folder-v2 | 1ed732d3-1461-4659-ab01-af50f58e54f7 | Official compares thick folder and thin booklet; both fit hands/counter |
| crossing-bucket-v2 | cf2c4074-98d7-4275-8943-2b9bc6b42287 | Official works on papers on an inverted bucket; plausible crouch and reachable writing surface |
| crossing-concrete-v2 | af8f24ed-5993-42d7-82c7-a552bbec470d | Official indicates actual foundation crack; hands, crack, and counter remain unobscured |
| crossing-documents-v2 | f22b37bf-1f12-44c9-b8df-27a64fc0c363 | Two external figures, matching portrait documents and monitor; human-scale cards, no crew substitution |
| crossing-clinic-v2 | a1b7c12f-d43a-443d-b8b2-749e6824a718 | Official indicates empty chair; second official handles form; no religious/medical cross symbols |
| crossing-inspection-v2 | ef63ae16-fd09-4151-9c52-54fee85d7a51 | Crouched inspector reaches the actual van plate; one coherent ground plane and scale |
| crossing-slots-v2 | bed315bd-0090-46d1-ae5d-eb9774fe3c74 | Wide payment slot and narrow objection slit; separate readable live labels; no overlap on phones |

Previous review reported phone caption/van overlap and slot-label layout problems. Those are not evidence that replacement painting is necessary. The restored candidate must be inspected before choosing layout correction or user review.

## Next decision

Recover the missing worktree/catalog from a retained copy if available. Otherwise reconstruct only the needed asset bindings from recorded evidence, preserving ambiguous files for review. Then replace each unresolved item with a concrete reuse decision or a complete individual image brief. Present the finite list and numerical composition rules for review before any generation or resuming game implementation.

This draft does not claim the full image specification, feature, or release is complete. No generation budget is committed while the required image list remains unresolved.

## Verification for this planning pass

- `sonar analyze secrets docs/content-hit-list.json docs/content-requirements.md progress.md` ran successfully using existing keychain access and reported no issues before those workspace files were read.
- Original PNG headers provide dimensions; hashes identify unchanged originals. No visual acceptance inferred from headers, filenames, or historical test results.
- Main-checkout Hearing/game files were not edited. No builds, browser campaigns, generation calls, subagents, commits, or deployment in this planning pass.
