# Cast production record

These are design intentions, not demographic inferences from names or artwork. Stable persona IDs and gameplay traits are preserved. Player-entered names never determine pronouns. Use they/them for the player crew in English narrative; feminine presentation is compatible with that neutral narration. Translated prose should avoid assigning gender through a player name. The existing narrative pronoun audit is still pending, so this policy is not a claim that all current copy complies.

| Role ID | Intended appearance | Reused views |
|---|---|---|
| journalist | Feminine East Asian adult | Standard, happy, defeated, unwell, standing, near seat, far seat, driver |
| organizer | Feminine Black adult | Same eight views |
| whistleblower | Gender-neutral South Asian adult | Same eight views |
| lobbyist | Older feminine Latina adult | Same eight views |
| staffer | Feminine Arab adult | Same eight views |
| satirist | Gender-neutral Indigenous American adult | Same eight views |

Each crew member has one RGBA atlas in `dystrail-web/static/img/cast-v2/`. Independent transparent layers are required for road scenes. The common renderer owns crop coordinates; choosing a different scene must not introduce a different identity. The active party controls presence and replacement driving.

Supporting portrait cells in `supporting-cast.png`:

| Cell | Fictional role and design intention | English reference |
|---|---|---|
| 0 | Feminine Black park ranger with unused work gloves | the ranger / they |
| 1 | Gender-neutral East Asian mechanic with unused switch | the mechanic / they |
| 2 | Feminine South Asian printer with identical sample sheets | the printer / they |
| 3 | Feminine Indigenous American medic with unopened pouch | the medic / they |
| 4 | Gender-neutral Afro-Latina local worker with closed lunch bag | the worker / they |
| 5 | Older feminine Arab librarian with closed book | the librarian / they |

Supporting sheets currently have opaque portrait backdrops. Use them as framed portraits only; they are not transparent scene sprites. Their pixel style needs review against the revised environment standard. Hearing chair/clerk use a separate framed mood atlas and the hearing feature's actual report.

Appearance audit:

- Integrated: character selection, crew naming, standing road crew, van occupants and replacement driver, encounter/care speaker, roster, hearing arrival and rounds, result portraits, downloaded share-image portrait.
- Still required: homepage art and screenshots; all fictional NPC assignments; translated pronoun continuity; contextual held props after completed actions; supporting-art style consistency; a final visual sweep across saved and offline states.
- Old artwork remains in the package for update compatibility. An old asset's presence on disk is not evidence that the active scene uses it; inspect the renderer and derived page images separately.

Asset hashes and exact generation prompts are in `review/art-satire/asset-catalog.json`. The first four-setting environment atlas was rejected for realistic rendering and has a separate replacement review record.
