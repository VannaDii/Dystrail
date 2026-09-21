# Policy announcement integration

Implemented in the isolated visual-world feature on September 14, 2026. This is an in-progress feature slice, not a release approval.

All eighteen A/B/C policy bulletins now have a full graphical presentation across shutdown, road restrictions, book panic, tariffs, education closure, and military rebranding. Their titles, activation prose, expiration prose, and factual source hooks follow the saved source unit. All effects and remaining duration come from the existing engine/HUD readout. The renderer does not apply policy effects.

The illustrations are separate editorial views, rather than assertions that the travelers have entered the pictured office or ceremony. They contain fictional locals and props, with no baked-in player crew. The military scene does not open the player's hood or invent a breakdown. The checkpoint image does not create another crossing decision. This adapts the source illustrations to actual game state. Required wording is editable beneath the illustration; it is not embedded in the raster image.

Each of the three atlases uses six cells in engine order, with exact generation and correction prompts retained in `review/art-satire/prompts/`. The A generation's masculine official/background guests were corrected. The B vending-machine column was corrected to block the small coin return beneath the keypad, leaving the snack pickup opening clear. Source and built asset bytes are checked during review.

## Persistence and action handling

`visual_content.policy_bulletins` is an additive saved array. Each record contains a stable engine-log-position ID, order ID, selected source unit, received day/time, and acknowledgement flag. Old saves default to an empty array; an already active order in an old save is not treated as a new activation.

The common completed-action journal path examines only appended `exec.start.*` records. That captures travel and rest/work activations, including an order that starts and ends within a longer completed action. The displayed timestamp says **received**, since the bulletin can be presented after such an action ends. Expired events explicitly say they have ended rather than promising a continuing effect.

Travel cannot advance while an unacknowledged bulletin is visible. A completed travel step is saved before its animation finishes, including its pending bulletin. Reload therefore restores the already-applied event rather than rolling it again. Manual export/import carries the same presentation record. Acknowledgement changes only that record; it does not spend time, adjust stats, roll randomness, clear care/encounters, or replay a grant/penalty.

New edition-one activations select A/B/C with an independent seed/family/occurrence hash. Existing recorded selections stay fixed; legacy and unknown editions select A. Activation and expiration journal messages follow the same occurrence, including a bulk action that starts and ends a policy. The current status-bar title and help follow its saved variant after acknowledgement. Imported unknown or cross-family unit references fall back to the correct family's A presentation.

Source hooks also follow the selected unit. The architecture variant uses the federal architecture order rather than the unrelated school-order source. The shutdown C food-assistance hook uses the verified CRS account of the reconciliation law, explicitly distinguishing that law from a government shutdown. This replaces an unavailable workshop AP link while preserving the workshop's stated hook. The fictional owner's English pronoun changes from “his” to “their”; feminine localized owner wording matches the illustration.

After the last bulletin, the existing outcome or pending action remains available. An already scheduled map preview is retained. Ordinary travel can resume after the player presses Continue. The bulletin heading receives focus after an import drawer finishes closing; the drawer otherwise restores focus to its own trigger.

## Current coverage and remaining production

- A/B/C art and editable labels are integrated for all six policies. Narrative/overlay coverage is English, Spanish, Italian, and Arabic; the other sixteen offered languages retain explicit English narrative fallback keys. The three new functional bulletin labels have all twenty interface translations.
- Native checks exercise once-only capture, duplicate acknowledgement, old saves, expired events, and an independent 100-seed engine/RNG comparison. Browser checks cover all six illustrations in four languages on desktop/phone, manual save/import, offline reload, preserved care/encounters, and real Classic/Deep travel activations.
- B/C selection is enabled. Browser coverage checks all twelve added units in four languages on desktop/phone, selected source links, manual import, offline reload and preserved care; targeted final-art review also checks the active status-bar help. The larger feature's remaining road/care/town variants, scene staging, derived art, and final release/update checks remain open.
- No engine or hearing rules were changed by this slice. No deployment has been performed.

Evidence is retained under `review/art-satire/policy-*`; current summary: `policy-bc-review.json`. Earlier focus and test-helper failures are preserved separately; passing evidence must be read against its recorded build revision.
