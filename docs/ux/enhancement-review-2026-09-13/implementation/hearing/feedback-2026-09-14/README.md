**Hearing playtest feedback — implemented September 14**

[Replay from one turn before arrival](http://127.0.0.1:62524/play/before-hearing-review.html). This opens a test crew in the updated local preview. Click **Travel**, then **Enter the hearing room** when ready. The existing preview on port 62523 has also been refreshed. The [portable save](../before-hearing-save.json) remains usable through Menu → Save / Load → Restore from file.

- Arrival stays until the player advances, including Fast and reduced-motion settings.
- The speaker, remaining crew and officials have framed portraits with visible names. The detached microphone icon is removed.
- Preparation now speaks through short dialogue and crew observations. Supply/cash preparation remains disclosed through the story; numeric rules, survival forecasts and the mechanical help panel are removed.
- Each round records Influence, Sanity spent with before/after values, and Odds with their before/after change in the shared StatCard component. These receipts precede Continue.
- The committee’s “No further questions” announcement stays in the hearing history and precedes the vote action. It also survives reload and appears in the scorecard. Exhausted crews do not receive this announcement.
- The closed hearing offers one action, **Call the vote**. Earlier **Skip to verdict** skips presentation; completed outcomes still lead to **View scorecard**.
- The average/multiplier tutorial and exact final-draw explanation are removed. The × and ≈ operators are larger and readable on narrow screens.
- All six crew identities have happy and defeated portrait variants. Verdicts, result profiles, crew epilogues and exported sharing images use the appropriate expression. Departed/dead companions keep their separate fate presentation. Standard art is preserved.

The hearing’s probabilities, sanity costs, continuation decisions, original final draw, policy guarantee and once-only resolution are unchanged by this presentation revision. The previously accepted 197/1,000 Classic/Balanced result remains the model’s existing campaign evidence; this revision does not retune or rerun that campaign.

**Review the screens**

[Arrival](screenshots/arrival.png) · [Preparation](screenshots/preparation.png) · [Opening argument](screenshots/opening.png) · [Verdict](screenshots/verdict.png) · [Scorecard](screenshots/scorecard.png) · [Sharing image from this replay](screenshots/share-real-run.png)

[Phone round](screenshots/hearing-round-1-mobile.png) · [320px layout with long names](screenshots/hearing-narrow-chromium.png) · [Arabic](screenshots/hearing-ar-mobile.png)

**Validation**

54 desktop/mobile browser checks, 378 native tests, and 2 WebAssembly browser tests pass. Native and WebAssembly pedantic lint, formatting and whitespace checks pass. All 128 offline assets match their manifest lengths and SHA-256 hashes. Release revision: `d085be895d332cd717a9`.

The tests cover held arrival, all four endings, staged stat disclosure, save/load/reload, Fast, reduced motion, narrow layouts, four languages, and offline use. Every persona’s exported avatar is compared pixel-for-pixel with the correct atlas cell for both outcomes. The [happy](screenshots/share-happy.png) and [defeated](screenshots/share-defeated.png) PNGs are isolated forced-outcome fixtures for that check, rather than complete journey examples.

The supplied game skill client confirmed that the saved road checkpoint reaches Arrival in one Travel turn and remains there. A separate browser replay exercised Arrival → Preparation → opening result → verdict → scorecard and downloaded its sharing image, without runtime errors. Screenshots and the twelve expression variants were visually inspected.

[Validation](validation.json) · [Browser log](checks/browser.log) · [Native log](checks/native.log) · [WebAssembly log](checks/wasm.log) · [Offline integrity](checks/offline-integrity.json) · [Replay evidence](checks/preview-flow.json) · [Art prompts and provenance](asset-provenance.json)

The six new two-cell atlases live in `dystrail-web/static/img/journey/occupant-*-expressions-v1.png`. They were created with the built-in image tool, using each existing portrait as its reference, followed by a background-only edit. Atlas cropping happens in the game renderer and sharing canvas. No manual bitmap edits were used.
