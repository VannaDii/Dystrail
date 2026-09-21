# Dystrail fresh satire draft — revised 14 September 2026

The further September 14 diversity revision replaces 246 narrative packages within the existing 567. The pack now uses 238 distinct news hooks, with no specific hook appearing in more than six packages. Every one of the 57 Deep End road variants has a different hook; 13 new trans-focused stories cover counterterrorism rhetoric, care, military service, identity records, schools, employment and crisis support. All narrative packages retain a political premise, documented factual basis, and scene direction. The four locked approved examples remain exact. Of the 77 factual and functional records, only the Deep End mode description changes. Supply quantities remain in mechanical data; player prose uses ordinary terms such as some food and some water. This remains an English editorial draft for review.

The review surface remains the existing [Dystrail Satire Workshop Google Doc](https://docs.google.com/document/d/1YIwegfefxIMTskhFsbTSwHBmWNyGo9zCmANuKfvLf9k), with 16 tabs and native version history. This pass saved revisions to 77 narrative units plus the mode description and opening guide. A normal connector retry under renewed user authorization saved 15 more units; automatic approval review then rejected further writes again, citing political satire rather than neutral factual content. The complete revised pack is local; 172 units still need synchronization, including three final refinements to previously saved units. The saved checkpoint passed native text, source-link and formatting checks. Read the live document and preserve team edits before retrying; do not treat the local draft as fully saved to Google Docs.

## Coverage

- 231 roads: 72 Classic, 57 Deep End, 102 Shared; 669 ordered choice branches.
- 132 town conversations: three distinct sourced angles for each of 44 playable towns.
- 24 care scenarios, 18 outside-ally departures, 12 crossings, 18 executive orders, 12 repairs, 24 routine activities, 27 weather and health conditions, 18 character introductions, 6 hearing packages, and 45 endings.
- 77 reference records: 51 town profiles, 7 endpoint contexts, 11 shop items, 6 personas, and 2 modes.
- 644 scene directions, including the reference records. Required words belong in separate editable overlays, never inside rendered images. Cast, pre-choice staging, and state-dependent variants are recorded with the scenes.

## Files

- `complete-pack.json`: all 644 records with copy, scene directions, references, and integration metadata.
- `narrative-pack.json`: the 567 narrative records only.
- `roads-a.json`, `roads-b.json`, `towns.json`, `support.json`, `functional.json`: authored source collections.
- `doc-sections.json` and `doc-sections/`: the complete intended review text and paragraph roles, including updates not yet saved to Google Docs.
- `validation-structure.json` and `validation-political-revision.json`: earlier coverage and revision snapshots, retained for provenance.
- `validation-deep-revision.json`: the earlier 97-package revision snapshot, retained for provenance.
- `validation-diversity-revision.json`: current counts, all 246 revised narrative IDs, ordered mechanics, approved-copy preservation, scene requirements, hook budgets and mode-boundary decisions. It records 63 Deep-only packages. Humor remains an editorial judgment.
- `political-sourcebook.json`: the expanded factual hook bank; individual units retain their specific source qualifications.
- `doc-save-verification.json`: native readback of the saved checkpoint, confirming every paragraph, all 644 unit headings, source URLs and font sizes. It explicitly records the partial save and remaining units.
- `doc-saved-sections.json`, `doc-diversity-sync-checkpoint.json` and `doc-pending-updates.json`: the verified saved text, revision checkpoint and remaining unit IDs for resuming the same document safely.

The mobile review retains playable copy, brief scene direction and linked source labels. Repeated art-state instructions and detailed source qualifications remain in the structured records.

All 644 IDs match `docs/content-hit-list.json`. There are no missing packages or structural blockers. Existing road families retain ordered mechanics; the 12 new western families have proposed mechanics that still need design review. Source facts and fictional commentary remain distinct. Retained population and reference facts keep their dated basis.

## Integration details to retain

The September 14 mode revision reserves the heavier stories about attacks on trans rights, reproductive restrictions, stalking and surveillance abuse, denied healthcare, and severe state coercion for Deep End. Its 57 road variants and six crossing/hearing variants carry `content_gate: deep_only` plus explicit topic metadata. The mode-selection copy names these themes before the player opts in. Shared content must meet Classic's lighter tone. Trans identity and representation are not themselves heavy material: the retained Salt Lake City story about adopting official Pride flags remains Shared, with that editorial decision recorded in validation.

Keep the gates when integrating: the current road selector treats an empty mode list as Shared. Never put a Deep End unit in that empty-mode pool, reuse it as a Classic fallback, or show its scene through a shared support package. The six Deep End crossing/hearing records need the same explicit selection guard. The metadata and review document do not themselves change the deployed game.

The old executive-order editorial slots map to the actual simulation keys: `ORDER-MILITARIZE` → `travel_ban_lite`, `ORDER-GAG` → `book_panic`, `ORDER-TARIFFS` → `tariff_tsunami`, `ORDER-TAXCUTS` → `doe_eliminated`, and `ORDER-DEREGULATE` → `war_dept_reorg`. Shutdown remains Shutdown. These mappings are explicit in the source records.

The shop's legal-fund item grants credibility but does not currently carry the inventory tag checked for tariff protection. The copy does not promise protection merely from that purchase. This draft does not change the runtime behavior.

Care follows the current traveler, including player-specific loss. Outside allies are contacts rather than van passengers. Repairs preserve the cashless radio-work path. The hearing has three stamina rounds followed by one vote if endured. Endings do not invent casualties, arrival, or a hearing where the recorded cause does not establish them.

After English approval, integrate approved copy and scene assignments, produce images, update Spanish, Italian, and Arabic translations, and verify the resulting game behavior. This writing task made no runtime or production deployment changes.

Authoring scripts are retained for provenance. Some apply earlier stages of the draft; rerunning them indiscriminately can overwrite subsequent editorial refinements. The final JSON collections and the live reviewed Doc are the current draft sources.
