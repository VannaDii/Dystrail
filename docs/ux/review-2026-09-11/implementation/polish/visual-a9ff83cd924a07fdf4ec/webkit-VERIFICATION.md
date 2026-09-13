# Final unit5 WebKit visual audit

Revision: `a9ff83cd924a07fdf4ec`.
Artifact: `/tmp/dystrail-visual-refine/unit5-web`.
Read-only preview: `http://127.0.0.1:63070/play/`.
Runtime: explicit Node 24.19.0; Playwright WebKit from `/tmp/dystrail-browser-engines-pinned`; disposable contexts only.

Final result: PASS. The HUD harness, final store/focus harness, and actual store-click harness all exited 0 with no page errors. No game source, server, production, or user-profile changes were made.

## HUD and help

The supplied HUD harness exercised 20 combinations: Classic/Deep × Clear/Cold Snap with Book Panic × 320/393/430/600/1440 CSS-pixel widths. Every case reports no horizontal overflow. All eight 320/393 HUD screenshots and four 393px weather/policy help screenshots were visually inspected.

- Date/time, pace, diet, town, cash, vehicle, weather and policy remain readable and visible. Weather and policy form one row when both apply.
- Theme colors are correct for Classic and Deep; no teal focus color remains.
- Weather and policy help popups are contained, readable and retain reachable close controls. Costs/durations remain in those details instead of cluttering the compact HUD.
- The 320px header wraps Menu into a second row (header bottom 108.19px versus 73px at 393). This is responsive wrapping, with no clipping.

Measured HUD sizes are identical between modes:

| Width | Condition | Resource HUD height | Conditions HUD height | Trip-context row |
| --- | --- | --- | --- | --- |
| 320 | Clear | 111 | 100 | 304 × 30 |
| 393 | Clear | 111 | 70 | 377 × 30 |
| 320 | Cold Snap + Book Panic | 111 | 100 | 304 × 30 |
| 393 | Cold Snap + Book Panic | 111 | 100 | 377 × 30 |

## Compact store and focus

Both modes were checked at 393 and 320px. All 11 item images decode, every card and quantity control remains within the viewport width, and all 132 settled control hit tests pass.

- At 393px, cards are 317 × 103px; checkout is x38/y716/w317/h136, ending at viewport bottom852.
- At 320px, cards are 244 × 103–115px; checkout is x38/y704/w244/h148, also ending at852.
- Store help is x61/y145/w320/h214 at393 and x12/y180.19/w296/h214 at320, entirely contained.
- Focus is genuinely `:focus-visible`, 3px solid: Classic `rgb(243,206,105)`, Deep `rgb(241,177,211)`. Final screenshots keep the whole focused Menu button visible.

Actual WebKit clicks on the last catalog item at320 succeeded in both modes: Legal Fund quantity0→1, cart total$71→$86, cash after purchase$49→$34. Remove restored quantity0, total$71, cash$49. Checkout's ordinary Playwright trial click passed. At the final-item position, card x38/y473.19/w244/h115 ends588.19; checkout starts602.19, leaving14px clear space.

## Resolved temporary-harness failure

The first supplemental DOM hit check sampled while CSS smooth scrolling was still moving the page. It failed before stable geometry. Original failure output and screenshot are retained in `reachability-initial-failure/`. Requesting `behavior:'instant'` only in the temporary harness produces132/132 visible/uncovered controls. Normal Playwright add/remove clicks then confirm real interaction; no application overlap defect was found or hidden, and no source was changed to obtain the pass.

## Evidence

- `report.json`: 20 HUD cases and page errors.
- `extra-report.json`: 4 store/focus/help cases,132 settled control hit results.
- `click-report.json`: real last-item add/remove and checkout reachability, both modes at320.
- `classic/deep-clear/policy-320/393.png`: HUD screenshots.
- `classic/deep-weather/policy-help.png`: weather/policy help screenshots.
- `classic/deep-store-320/393.png`: full-page store captures. Sticky checkout appears once at its current viewport position; final click captures show actual reachability.
- `classic/deep-store-help-320/393.png`: store help.
- `classic/deep-last-store-item-click-320.png`: actual changed quantity and reachable checkout.
- `classic/deep-focus-final-320.png`: visibly correct focus rings.
- Process logs are alongside this directory: `webkit-unit5.log`, `webkit-unit5-extra.log`, `webkit-unit5-click.log`.

This is a bounded English WebKit visual/browser check, not a claim about every locale, device, or native Safari version. Root independently owns the full Chrome/native verification and release decision.
