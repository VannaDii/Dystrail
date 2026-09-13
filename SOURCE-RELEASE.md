# Complete source and desktop status release

Source: VannaDii/Dystrail branch `release/complete-game-source-2026-09-13`, runtime commit `2285204419c3dd3055a0fbdf7f763aa74f386b07`.

Game revision: `837bb4f404afd0f3f7fc`. The desktop status row no longer reserves an empty help-button slot; visible readouts have consistent spacing. The mobile grouping is retained. The homepage screenshot shows the current shipped interface and removes its stale Pants reference while preserving the approved Play in browser / …even offline! treatment.

The release uses the unchanged verified game runtime from `d3ed78dac458b64a80ee` and the new source stylesheet. This preserves existing immutable URLs despite nondeterministic generated-closure ordering during an otherwise identical runtime rebuild. The complete game source, tests, writer content and evidence are now committed and pushed in the source branch. The independent kernel work on main is preserved.

Validation: seven viewport widths in both modes, eight existing status/help tests, inspected browser screenshots, the original game browser client, and an existing-install update with all 119 offline assets and the full save preserved through reload and closed-browser offline restart. The complete source also undergoes the CI checks on its release branch before publication.
