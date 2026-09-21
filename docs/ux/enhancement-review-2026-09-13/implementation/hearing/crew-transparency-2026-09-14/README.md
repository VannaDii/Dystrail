# Standing crew transparency repair — September 14, 2026

Corrected the standing-sprite chroma key that left translucent magenta rectangles around all six crew members in the in-app browser. Existing character artwork, party membership, positioning and hearing rules are unchanged.

The defect was reproduced in the actual affected browser. Moving the filter into each SVG or explicitly setting primitive color spaces did not remove the matte. Increasing the chroma-key separation removed it while retaining the original RGB rows, outline erosion and costumes. Side-by-side inspection covered all six full-size sprites; the actual road and arrival scenes were then inspected in the in-app browser.

Build `a9965b9895e256c1042c` is served on local previews 62523 and 62524. The user's current 62524 tab was reloaded and visually confirmed clean at its existing 638 px width. It resumed the same stopped-road state: Day 2, 10:02, six members, stats 13/9/10/7/0/5/1 and $14. No browser errors or warnings were reported.

Validation:
- Four focused desktop/phone cases passed in the bundled Chromium headless shell; the same four passed in installed Chrome.
- New rendering regression samples actual composited pixels around all six sprites at 128, 64 and 41.5 px widths. Every sampled margin matches the backing exactly. Existing crew absence and save-recovery assertions also pass.
- The supplied game client captured stopped road and Arrival after a Travel action. Screenshots and state JSON are under `game-client/`; no runtime errors were recorded.
- Strict Wasm Clippy, Rust formatting, whitespace validation and all 128 offline asset hashes/lengths passed.
- No gameplay or probability changes; no commit, PR or production deployment.

Evidence includes the reported image, desktop/phone crew screenshots, and the game-client road/arrival captures. Native in-app screenshots were inspected directly in this conversation; they are not represented as headless captures in this folder.
