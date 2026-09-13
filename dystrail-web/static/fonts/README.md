# Bundled game fonts

These unmodified WOFF2 files are shipped with the game. There are no external font requests at runtime. All seven declared faces are cached, checked against the offline manifest, and loaded before the loading screen closes.

| Family | Shipped faces | Use |
| --- | --- | --- |
| Atkinson Hyperlegible Next | Normal and italic variable, weights 400–800 | Humanist body text and controls |
| Courier Prime | Bold 700, Latin and Latin Extended | Typewriter headings |
| Noto Sans Arabic | Normal variable, weights 400–800, Arabic/Latin/Latin Extended | Arabic text and fallback |

The font files total 334,004 bytes. Google Fonts' separate mathematical and pictographic Noto subsets are not included: game symbols use the existing icon components, and the bundled faces cover the game's Arabic and Latin text. The original files were not modified or recompressed.

## Sources and licenses

- [Atkinson Hyperlegible Next publisher repository](https://github.com/googlefonts/atkinson-hyperlegible-next), commit `7925f50f649b3813257faf2f4c0b381011f434f1`; original variable WOFF2 assets and [license](atkinson-hyperlegible-next-OFL.txt).
- [Courier Prime in the Google Fonts repository](https://github.com/google/fonts/tree/main/ofl/courierprime); official Google Fonts v11 WOFF2 assets and [license](courier-prime-OFL.txt).
- [Noto Sans Arabic in the Google Fonts repository](https://github.com/google/fonts/tree/main/ofl/notosansarabic); official Google Fonts v33 WOFF2 assets and [license](noto-sans-arabic-OFL.txt).

All three families use the SIL Open Font License 1.1. The complete copyright notices and licenses are kept alongside the fonts. [provenance.json](provenance.json) records the exact download URLs, pinned license sources, sizes and SHA-256 hashes, retrieved on 2026-09-13.

## Integration

Load `static/fonts.css` as a normal stylesheet without relocating it; its URLs are relative to `static/`. Set body text to `"Atkinson Hyperlegible Next", "Noto Sans Arabic", sans-serif` and headings to `"Courier Prime", "Noto Sans Arabic", monospace`. Arabic may put `"Noto Sans Arabic"` first. The theme owns these font stacks and sizes.

`offline-client.js` reads every loaded `@font-face` declaration, finds its exact source in the verified release cache, checks its length and SHA-256, and creates a `FontFace` from the cached bytes. Only after every face loads does it register them and remove their equivalent URL-backed CSS rules. This prevents unused weight, italic, or Arabic faces from causing later requests. Any missing declaration, unavailable asset, bad integrity, or font-decoding failure retains the loading screen and its retry control. The existing artwork preparation still completes before launch.

`window.dystrailPreparedFonts` holds all seven loaded `FontFace` objects for the document lifetime. Browser verification can inspect their family, style, weight, Unicode range and `status === "loaded"` before `window.dystrailLaunch` resolves. The offline manifest generator automatically includes the font files, this stylesheet, provenance, and licenses.
