# Independent encounter visual verification

Verified candidate: `2fa3864f31cbf9af2ff6` at `http://127.0.0.1:63070/play/`.
Frozen artifact: `/tmp/dystrail-visual-refine/unit3-web`.

Result: all 24 browser combinations passed, and all 24 full-page screenshots were visually inspected. These findings apply to unit3; the parent's subsequent focus-color-only unit4 is outside this report.

## Coverage

Six encounter settings, Classic and Deep, at 1440 x 1050 desktop and 393 x 852 phone viewports. Disposable Chrome contexts only; source and production were not modified.

| Encounter | Scene | Atlas cell | Result |
| --- | --- | --- | --- |
| deep_secure_line | enc-motel | 0 | All four combinations pass |
| sat_press_pool_radio | enc-cafe | 1 | All four combinations pass |
| deep_field_intel | enc-library | 2 | All four combinations pass |
| sat_museum_grant | enc-museum | 3 | All four combinations pass |
| sat_corn_bullets | enc-farm-office | 4 | All four combinations pass |
| beltway_briefing | enc-service-counter | 5 | All four combinations pass |

## Observed results

- Motel, cafe, library, museum, farm office, and service counter show the correct distinct interiors. Desktop crops preserve recognizable props; phone crops show the broader room without neighboring atlas-cell bleed. No unexpected people are painted into the backgrounds.
- Scene titles remain legible over the image gradient/shadow, wrap cleanly on phone, and do not collide with the portrait.
- Exact narrative and all choice labels match the frozen encounter data. All required choice buttons render before the optional Trail history, with no horizontal overflow or clipped choice controls.
- The fixture has four active crew members, one departed, and one dead. It deliberately requests the departed member as the scene subject. Every rendered portrait belongs to an active member; departed/dead portraits do not render.
- The source help was opened in all 24 cases. Each popup stays inside the viewport, keeps the close control and source link visible, and inherits the correct mode. Measured main-copy contrast: Classic [13.09]:1; Deep [12.53]:1.
- Deep high contrast was additionally inspected using the longest English narrative (`classic_union_blockade`, 348 characters). Source help uses black on white (21:1), is fully visible on phone, and closes with Escape. The long narrative and choices wrap without clipping.
- Fresh loading gates in both modes close only after all seven font faces are loaded and all 68 images are decoded. All scene and source-help checks then run offline. No network asset downloads occur after gate removal; no page errors were recorded.

## Content follow-up, intentionally unchanged

Fixed headcounts conflict with reduced-crew saves. `beltway_briefing` choice 3 says “Split the packet between the six of you” in the four-active-member fixture. The extra long-narrative check also shows fixed “six people” / “six more people” wording in `classic_union_blockade`. Parent explicitly instructed that writer wording remain untouched in this visual release. These are content follow-ups, not atlas or portrait-filtering failures.

## Evidence

- Machine report: `report.json` (24 cases, source geometry/contrast, loading-gate facts, no network assets after gate, no page errors, passed true).
- Reproducible temporary harness: `verify.mjs`.
- Full pages: `{classic,deep}-enc-{motel,cafe,library,museum,farm-office,service-counter}-{desktop,phone}.png`.
- Individual decision panels: matching `-decision.png` files.
- Source popups: matching `-source.png` files (all phone and Deep desktop).
- High contrast: `deep-high-contrast-long-source-phone.png` and `deep-high-contrast-long-encounter-phone.png`.

This is a bounded Chromium visual/loading check. It does not replace the independently running full browser suite, native tests, accessibility audit, or Safari verification.
