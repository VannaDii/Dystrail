# Compiled result/share verification — unit 3

Candidate: `http://127.0.0.1:63070/play/`, revision `2fa3864f31cbf9af2ff6`, confirmed from the active offline client in all 24 result/share layout snapshots. Disposable Chromium contexts only; no user profile, source, build or production changes. No external social-platform navigation or posting occurred.

## Verified behavior

- Classic and Deep, English and Arabic, at 1440×1000, 393×852 and 320×568: all 12 result statistics render in the shared boxed component, with no document overflow or card extending beyond the viewport.
- The fixture's 42 elapsed days, 12 encounters, 3 receipts, 5 allies, 16 supplies, 7 credibility, one breakdown, 1,524 physical miles and two starvation days retain their exact localized values. Scores and threshold status remain present as separate cards. The native regression separately covers both threshold outcomes.
- All three share actions (copy, image download and native-share handoff) stay fully inside every viewport without scrolling through the preview/editor. The desktop image/editor split remains intact; 320px uses a smaller image with an internally scrollable editor area and persistent action footer.
- Close receives initial focus. Shift+Tab wraps to the final share action; Tab returns to Close. Mouse close and Escape restore focus to the result share opener.
- Editing the post updates both platform draft URLs and the copied/native-share payload exactly. Clipboard and native-share methods used isolated adapters, so the test did not alter the user's clipboard or invoke an external share destination. Native cancellation correctly releases the busy state.
- Four actual image-link downloads succeeded after the browser context was set offline. PNG signatures, file sizes and 1200×1200 dimensions were checked; `downloads.json` records hashes.
- Actual pixels in each generated PNG match the active `--panel`, `--panel-raised` and `--accent` theme colors. Deep output uses purple/pink; Classic uses blue/gold. Canvas font assignments consume Courier Prime, Atkinson Hyperlegible Next and the Noto Sans Arabic fallback from the theme. All four downloaded PNGs were opened and visually inspected for readable text, correct mode treatment and uncut content.
- Additional 320px crew captures confirm the new portrait-above-role layout: every role label is one line, including Organizer and Whistleblower. The empty crew-name fixture keeps Continue disabled as expected; this capture was for role/name tile layout.
- No page errors. No remaining issue found in this scoped verification.

## Evidence

`metrics.json` contains the 24 layouts. `*-verification.json` records each PNG palette/font check and copy/share payload check. `*-download.png` are the actual downloadable images; `downloads.json` records their size/dimensions/hash. `*-crew-labels.json` records one-line role labels. Representative screenshots: `deep-share-ar-narrow.png`, `deep-share-en-desktop.png`, `classic-result-en-desktop.png`, `deep-result-en-narrow.png` and `deep-crew-narrow.png`.

The reproducible script is `audit.cjs`; it exited successfully. The application source remained frozen throughout.
