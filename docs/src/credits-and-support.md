# Credits & Support

The game provides an optional **Credits & Support** modal from the title/setup screen, the in-game Menu, and the end-of-run screen. It is available without completing a run and never opens automatically. Donations do not unlock game content or affect a run.

## Credits and acknowledgments

The modal identifies Vanna DiCatania as the creator, links to the project's contributor history, and acknowledges that the artwork includes AI-generated illustrations. The current game has no audio playback or bundled music requiring audio credits. Update the acknowledgments if that changes.

The source package declares MIT licensing. Geographic data has separate terms: boundaries come from the U.S. Census Bureau, and routes derive from OpenStreetMap under ODbL. The modal preserves those distinctions; the existing map-scene attribution remains in place. See [geographic input provenance](https://github.com/VannaDii/Dystrail/blob/release/complete-game-source-credits-2026-10-04/docs/ux/map-sources/README.md).

Atkinson Hyperlegible Next, Courier Prime, and Noto Sans Arabic use SIL Open Font License 1.1. The modal links to the full license notices shipped beside the fonts. See [font provenance](https://github.com/VannaDii/Dystrail/tree/release/complete-game-source-credits-2026-10-04/dystrail-web/static/fonts).

## Democracy support

The selected beneficiary is **Democratic Socialists of America (DSA)**. The link opens its [standalone donation page](https://act.dsausa.org/donate/donation), rather than its membership form. Donations go directly to DSA and are separate from creator support. The game does not collect payment information or claim affiliation with DSA.

The destination was reviewed on October 4, 2026. It offers one-time and monthly donations and links separately to membership. Its published form contains no citizenship affirmation; international payment completion has not been qualified. DSA's [privacy policy](https://www.dsausa.org/privacy-policy/) permits newsletters using a supplied email address, with an opt-out afterward. Vanna accepted that email policy. DSA describes these donations as not tax deductible.

Check the destination again before changing it or making new claims about donor eligibility. A donation destination that requires political membership or US citizenship/residency does not meet the approved beneficiary criteria.

## Creator support

The **Support the game** section links to [Vanna's Buy Me a Coffee page](https://buymeacoffee.com/vannadi), using the destination Vanna supplied on October 4, 2026. It explains that optional tips support her work on the game. This payment destination is separate from the DSA donation link.

The approved HTTPS URL is stored in `COFFEE_URL` in `dystrail-web/src/components/ui/credits_support.rs`. Rebuild the game and offline bundle after changing it. No build-time environment variable is required.

## Interaction and offline behavior

The browser-native modal makes the underlying game inert, manages keyboard focus, and closes with its Close button, Escape, or a backdrop click. Closing restores focus to the opener; when opened from the in-game menu, focus returns to the Menu button. Opening stops automatic travel and pauses the timed hearing. Closing does not restart automatic travel.

Links open in a new tab with `noopener noreferrer` so the game stays open. Credits and bundled font licenses remain available offline; external destinations require connectivity. No payment widgets or fundraising scripts are embedded in the game.

The new copy uses the existing translation system. English text is included in all locale bundles, matching the project's current convention for newly introduced copy; translations for this feature remain pending.

## Local verification — October 4, 2026

- The WebAssembly build and all 105 web-library tests pass, including locale-key parity.
- Native and WebAssembly lint checks and workspace tests pass.
- The documentation book builds successfully.
- Chrome checks cover title, menu, and results entry points; desktop and 390-pixel mobile layout without horizontal overflow; opening the modal while offline; keyboard activation, Escape, Close, backdrop dismissal, and focus restoration; and the new-tab attributes on support and license links.
- Vanna's coffee URL is connected in the source. Translations remain pending. Payment submission has not been tested.

## Publication status

The credits implementation and documentation are committed on `release/complete-game-source-credits-2026-10-04`. Publication is held because the full browser regression suite has existing mismatches with the currently published game. Native tests, native and WebAssembly lint, locale coverage, security checks, the optimized build, and the 1,000-campaign QA sweep passed. Direct credits checks passed against the CI artifact, including hearing pause behavior. See [release verification](https://github.com/VannaDii/Dystrail/blob/release/complete-game-source-credits-2026-10-04/docs/release/credits-support-2026-10-04.md).
