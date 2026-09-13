# Status notifications — 3c000988cf56b4dca6f1

New policy effects use the same brief highlight and reduced-motion treatment as weather changes. The shared notification survives scene transitions; ordinary duration/help updates do not restart it. Weather and policy pills retain 4px vertical and 8px horizontal padding throughout the highlight.

All six policy activations in both modes, notification expiry, unchanged-cost rerenders, help interaction, reduced motion, phone/desktop spacing and Arabic layout passed in an isolated browser. The first 85363a02b2a26488b4ce artifact supplied the full behavior checks; the final artifact retains identical status code/CSS plus the three already-verified nonbreaking speed labels. Both modes were checked again on the final artifact. No browser errors.

82 frontend native checks, formatting, and strict native/Wasm lint passed. Preview publication verified all 119 assets and the service worker. Real user saves and production were not touched.
