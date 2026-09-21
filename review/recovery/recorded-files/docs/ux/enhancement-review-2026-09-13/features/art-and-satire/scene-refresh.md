The September 14 editorial refresh changed eleven units that already had illustrations. Three replacement atlases supply twenty-two before/after frames while retaining the existing action mechanics, saved selection IDs and content modes.

| Atlas | Revised units | Source-specific staging |
|---|---|---|
| barter-refresh-20260914 | Tire B/C, Supplies B | Closed provisions bags and intact tires swap places. Pay slip and grocery receipt stay with their local owners. |
| work-refresh-20260914 | Food work A/B/C, Cash work B | Milk donations become sorted, dirty cookware becomes clean, a blank patriotic paper sleeve is discarded, and hall chairs become stacked. Rewards appear only after work. |
| gather-refresh-20260914 | Forage C, Glean A/B/C | Harvest baskets become full. Research paper stays with the botanist; tractor stays unrepaired; old wooden crate stays empty; corn/baseball comparison stays with the farmer. |

All art uses coarse pixel clusters, distinct silhouettes and feminine or neutral supporting residents. No player crew is baked into these atlases. Existing runtime portraits and current-roster filtering remain authoritative. The two farmers described with masculine pronouns in the workshop are feminine in the new art and localized copy; those two English pronoun changes are recorded in `review/art-satire/refresh-copy-adaptations.json`. Other English titles, setups, actions and outcomes match the pinned source.

English, Spanish, Italian and Arabic copy and source context accompany the revised scenes. Other offered locales keep their established interface and explicit English narrative fallback. The reused crate's stamp is native, editable SVG text with a matching screen-reader description. Its blank label was enlarged with the image tool; longer Latin translations need width control to fit without lowering their cap height. No labels are painted into raster files.

The three barter revisions passed the full barter/hearing browser suite, native checks and desktop/phone visual review on `2bdc917f124a677d4da0`. The four work revisions passed the activity matrices and native/Wasm lint checks on `b245d213eca63ef37080`. The gathering candidate `c5d3d8a410368eee9ac5` passed 397 workspace tests and strict native/Wasm checks. Its browser matrices exposed Spanish lettering that exceeded the crate panel, so gathering acceptance remains pending the corrected render. Exact combined acceptance evidence will be recorded in `review/art-satire/activity-refresh-review.json` when those checks finish.

These revisions preserve source ownership and game mechanics. The 36 proposed-mechanics records remain inactive. This refresh does not complete the remaining road, town, care, ally, repair, condition, departure, hearing and ending scene coverage, supporting/derived cast audits or final release verification. No commit or deployment is included.
