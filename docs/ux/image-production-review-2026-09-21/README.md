# Visual-world image production — approval package

**Status: approved by the user on 2026-09-21 for the bounded production and integration scope below.**

Worktree: `/Users/vanna/Source/Dystrail-worktrees/visual-world-recovery`. Local commits and standalone recovery bundles are retained outside temporary storage.

## Proposed scope

- Reuse the six retained feminine/neutral cast sheets, vehicle body/mask, suitable regional backgrounds and inspected scene cells.
- Correct seating, proportions, captions, lighting/weather, localized billboards and scene composition in code. Preserve approved UI/stat designs, actual crew, and Hearing reports/mechanics.
- Generate **at most 13 atlas images**, each **1536×1024**, four **768×512** scenes per atlas. These cover 52 selected visual compositions. One attempt per approved image; skip any image that becomes unnecessary. No automatic retries, renamed retries, separate mobile versions or per-locale paintings.
- Keep existing failed/mismatched candidates unused for personal review. The recovered freezer scene avoids a new attempt; the earlier freezer/leaflet mismatch is explicitly held, not regenerated.
- Preserve all 644 source records, with all 36 mechanics-dependent records inactive. Shared settings carry dialogue-led jokes; the selected visual compositions carry their essential actions.

Approval is for this bounded production and integration work. It does not authorize deployment, extra generations, new mechanics or a redesigned interface.

## Exact new-image list

| Image | Top left | Top right | Bottom left | Bottom right |
| --- | --- | --- | --- | --- |
| selected-satire-01 | ENC-C09-C / counter | ENC-C10-B / room | ENC-C10-C / room | ENC-C11-C / counter |
| selected-satire-02 | ENC-C12-C / counter | ENC-C14-A / outdoor | ENC-C16-B / outdoor | ENC-C16-C / room |
| selected-satire-03 | ENC-C18-B / outdoor | ENC-D01-C / counter | ENC-D04-C / counter | ENC-D06-B / counter |
| selected-satire-04 | ENC-D09-A / outdoor | ENC-D12-C / counter | ENC-D13-A / counter | ENC-S03-B / room |
| selected-satire-05 | ENC-S05-A / counter | ENC-S09-A / outdoor | ENC-S09-C / outdoor | ENC-S10-A / outdoor |
| selected-satire-06 | ENC-S11-A / outdoor | ENC-S13-A / counter | ENC-S14-A / doorway | ENC-S16-B / counter |
| selected-satire-07 | ENC-S16-C / doorway | ENC-S17-A / room | ENC-S17-B / outdoor | ENC-S18-A / room |
| selected-satire-08 | ENC-S19-B / room | ENC-S20-B / counter | ENC-S21-B / counter | ENC-S22-B / counter |
| selected-satire-09 | ENC-S23-A / outdoor | ENC-S24-A / room | ENC-S24-B / room | ENC-S24-C / room |
| selected-satire-10 | ENC-S26-B / counter | ENC-S27-B / room | ENC-S30-C / outdoor | ENC-S32-B / outdoor |
| selected-satire-11 | ENC-S33-A / outdoor | ENC-S33-B / doorway | ENC-S34-B / room | TOWN-01-A / outdoor |
| selected-satire-12 | TOWN-03-C / outdoor | TOWN-14-A / outdoor | TOWN-14-B / room | TOWN-23-C / room |
| selected-satire-13 | TOWN-29-C / outdoor | TOWN-34-C / counter | TOWN-35-B / counter | TOWN-44-A / counter |

Each panel’s source action, camera, principal subject/prop rectangles, contact line and caption reserve are specified in [scene specifications](scene-cell-review.md), under “Draft new-art portion,” and in `review/recovery/scene-cell-review.json` → `production_proposal`. Cells are independent scenes in reading order, with no gutters or art crossing their boundaries. A failed cell does not authorize a second attempt on another sheet.

## Reuse, dimensions and placement

| Surface | Dimensions and placement | Work required |
| --- | --- | --- |
| Six crew sheets | Each 1774×887; 4×2 cells; uniform logical rendering at 2048×1024 | Reuse expressions and body poses; preserve identities and actual roster |
| Van | 1536×1024 original with existing mask; six disjoint seat areas and explicit head/hand targets | Layout only; no new van or seating image |
| Regional roads | 1536×1024; coherent side-view van/crew composition | Reuse actual route region; exclude fixed old crew, religious imagery and incompatible baked lighting |
| Shared interiors | Exact 512-square or 512×256 atlas crops, contained at native aspect | Reuse only for a compatible setting; do not stretch to 3:2 |
| Hearing room | Retained 768×512 courtroom cell | Consume existing report/phases; actual arrivals only; no probability/round changes |
| Billboards | Native sign geometry; master face `[944,344,448,176]`, live text safe area `[968,360,400,144]` | Editable localized copy; zero generated sign images |
| Time/weather | Actual saved clock, shared presentation layers | No new time-of-day paintings, no indoor rain, UI above tints |

[Core layout specification](core-surface-review.md) supplies original paths, exact seat masks, pixel/camera/proportion rules and desktop/mobile placement. The JSON `shared_context_catalog` lists exact retained image paths and crop rectangles; `records` records every source disposition and explicit hold. These numerical specifications are proposals to validate in the running build, not a claim of visual acceptance.

## Desktop and mobile rules

Use the same composition and camera at every viewport. Scale whole scenes uniformly; no phone-only giant van or NPC. Preserve the crop’s native aspect. Keep captions over the image at bottom-left, with no supplementary bust. Verify at 1440px desktop and 390px/320px phones, including long/RTL text and reduced motion. Existing UI/stat components remain authoritative.

For the 3:2 new panels, a 390px-wide scene is 260px tall and a 320px-wide scene is about 213px tall. Essential action must remain readable above its caption. If composition or text fails review, fix layout where possible; do not automatically regenerate artwork.

## Held images and failures

The [retained-original gallery](image-inventory.html) contains all 176 originals for inspection. Its historical labels are not production approval; current decisions live in the linked specifications and JSON ledger.

Do not bind a candidate carrying religious symbols, wrong cast, incompatible staging, painterly style or broken proportions simply because it exists. A dialogue-led scene can use a suitable shared setting. A missing essential visual remains a visible acceptance issue for your review. No future generation is silently added to this list.

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


## Acceptance after approval

Recover and integrate preserved code before rebuilding it. Validate real scene rendering and surviving-crew continuity; preserve Hearing, localization, qualitative options, save/import, replay and offline updates. Run relevant checks after changes, not repeated broad tests on unchanged code. Deliver a committed release candidate, desktop/mobile preview sheets and an accurate report of every remaining issue. Do not deploy without separate approval.

**Approval received:** this bounded reuse/layout plan and up to 13 one-attempt atlas images. Existing holds and any new failures remain for your review; approving production does not waive the quality requirements.
