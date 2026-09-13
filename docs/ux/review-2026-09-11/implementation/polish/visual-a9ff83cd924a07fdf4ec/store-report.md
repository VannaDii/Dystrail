# Compact phone store verification

Verified staged CSS against immutable unit3 revision `2fa3864f31cbf9af2ff6` at http://127.0.0.1:63070/play/. The existing journey stylesheet was replaced in the same DOM position inside disposable browser contexts; no published artifact or browser profile changed.

## Changes
- Mobile item rows use 48px illustrations, 4px padding, and a 2px row gap. All 11 images remain distinct; existing 15px names and 12px descriptions remain unchanged.
- Quantity controls remain a separate row with 44px square touch targets. Removed the redundant 4px quantity top margin and reduced outer section spacing.
- The Arabic assertion now checks RTL horizontal order for overlapping rows, or verifies that a stacked quantity row begins below the complete name/description body. No cost, cap, help-jump, checkout, or height assertion was changed.

## Evidence
18 layouts passed: Classic and Deep, English/German/Arabic, at 412px Pixel 7, 393px, and 320px. Both modes produced identical dimensions per language/width.

| Locale | 412px height | 393px height | 320px height |
|---|---:|---:|---:|
| English | 1660.48px | 1660.48px | 1719.14px |
| German | 1660.48px | 1660.48px | 1743.20px |
| Arabic | 1665.64px | 1665.64px | 1678.67px |

The unchanged Pixel 7 `<1700px` guard is satisfied; English was 1984.48px before the fix. Longer text wraps naturally at 320px. There is no horizontal overflow or description/quantity/price overlap in any checked layout.

All 11 images decoded in each layout. All 66 quantity-control trial clicks at 320px and all 18 checkout trial clicks succeeded. No browser page errors were observed. English 393px and Arabic 320px screenshots were visually reviewed for readable copy, illustrations, quantity alignment, and distinct mode colors.

Files `metrics.json`, `errors.json`, `audit.cjs`, and 18 screenshots provide reproducible evidence. The audit replaces only CSS in an isolated context; root owns the compiled follow-up run of the unchanged shopping assertions. No Rust build or test was run here.

## Frozen file hashes
- `dystrail-web/static/journey.css`: `949805d2f819af62f7efbc0bb8d5623daaecd8590af8c95d33db26e6b13977c8`
- `dystrail-web/tests-e2e/shopping.spec.ts`: `0a17ad8c8038a8293a50b60bbc7d16364ea7b64faa7c7935dddc091e48b09543`
