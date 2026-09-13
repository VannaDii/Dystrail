# Save menu — a2666947301cbcb4bfb7

Compact device-save and backup sections replace the large form. File download and file restore are primary actions; copied backup text lives in a closed disclosure. Every action has its own pictogram, and the close control remains visible while expanded content scrolls. New interface text is localized in all 20 languages.

Browser checks cover backup file download/restore, rejected invalid files preserving the current state, legacy text import, disabled empty imports, keyboard wrapping and return focus, all 20 languages at 320px in both modes, and phone/desktop screenshots. All pass with no browser errors. Inventory tags are compared as the HashSet declared at dystrail-game/src/state.rs; ordered game data and all other state fields are preserved.

The first browser run caught a real focus bug involving closed disclosure contents; the focus filter now excludes those controls. Later fixture corrections account for existing pasted text and the Menu button that the header uses as its return-focus target. No gameplay experience guards were changed.

82 frontend native checks passed before the focus refinement; the final save-menu checks, formatting, strict native/Wasm lint and release build pass. Native Safari and physical-device file-picker behavior remain unobserved. This unit contains no game-engine or balance changes.

Published to preview 8180: all 119 assets and the service worker match a2666947301cbcb4bfb7. Real user save storage and production are unchanged. Clean phone review captures are included.
