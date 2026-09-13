# Frozen unit5 content inventory repair

Generated with `python3 scripts/build_content_review.py --source-root /tmp/dystrail-visual-refine/unit5-source`.

## Bounded changes
- The runtime encounter parser accepts both direct and braced string-valued match arms. It excludes the Rust test module and rejects duplicate, missing, or unknown encounter assignments.
- Removed the obsolete `raw_milk` override. It now follows the frozen runtime assignment to `enc-community`, as does the inventory CSV.
- Added the six `encounter-settings-v3` cells with their correct 3-column / 2-row layout. Thumbnail aspect ratios follow each actual crop, so the square interior cells are fully visible.
- Added the existing rest-area mapping with the runtime regional/default atlas choice; no thumbnail has an undefined cell.
- Reuse commentary now derives the busiest settings from actual counts instead of retaining the obsolete radio/civic claim.
- Editorial and source-check dates remain September 11, 2026. Generated inventory date is September 13, 2026.

## Validation
- **65 encounters / 187 choices / 51 towns / 18 scene compositions.**
- All 65 scene mappings independently match a separate line-by-line parse of the frozen runtime function, including all three IDs in the braced checkpoint arm.
- All authored encounter fields other than the deliberately derived runtime scene match source JSON. All town/source data is unchanged. The source JSON still contains some historical scene metadata; the rendered inventory correctly uses the runtime function, as before.
- All 187 CSV rows match exact labels, outcomes, effects, sources and assignments. HTML embedded inventory matches `inventory.json` exactly.
- Disposable browser rendering confirmed encounter/choice/town/catalog counts; all atlas images decode at 1536×1024; all 18 catalog crops are finite and in bounds. All six v3 names, atlas paths, cell coordinates and 512×512 dimensions match the frozen runtime helper.
- The six rendered interiors were visually inspected in `new-interior-cells.png`. Desktop 1440px and phone 393px layouts render without horizontal overflow or browser console/page errors.

Evidence: `data.json`, `browser.json`, `validate.py`, `browser.cjs`, `new-interior-cells.png`, `catalog-desktop.png`, and `catalog-phone.png` in this directory. No Rust build, game-source/asset modification, current-dist change, or deployment was performed.

## Frozen SHA-256
- `scripts/build_content_review.py`: `c5b4c983085ded1db59a5522f7e2941f93a5db71f54c3e9ca2be8de90ec1a3f5`
- `docs/ux/review-2026-09-11/implementation/content/index.html`: `970b916881cc630402618e3e12c4d44e9e709811f8744df6a80b3e1057fecdea`
- `docs/ux/review-2026-09-11/implementation/content/inventory.json`: `072fb0053b678264939b8d5cf54e35a1a06b712d836c362a712a1fa066f2ed7f`
- `docs/ux/review-2026-09-11/implementation/content/encounter-options.csv`: `d3eca866373b78c587c30cc33a9a90e5dcb350786257d4eae9b22dc8793b20dc`
