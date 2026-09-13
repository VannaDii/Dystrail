# Dystopian Trail website review

Implemented September 11, 2026. Local review: <http://127.0.0.1:8084/>.

The static homepage follows the approved desktop and mobile previews: original prairie and van artwork, immediate Play action, real gameplay capture, three gameplay descriptions, closing Play action, and documentation/source links. The two requested lines are present verbatim. The van drives from completely off-screen left into its position over 4.2 seconds, then stops. Reduced-motion preferences disable the entrance and interaction motion.

Product changes are confined to `site/`. This review directory records the website work. The existing game changes and deployment workflow belong to separate work and were not edited for the website.

## Search identity and metadata

The public identity is **Dystopian Trail**, with **Dystrail** as its alternate name. The canonical homepage is `https://dystrail.com/`. The page has a unique title and description, an English language declaration, crawlable HTML, Open Graph and Twitter metadata, a 1200 × 630 social image, a favicon, robots.txt, and a sitemap listing `/`, `/play/`, and `/docs/`.

A linked JSON-LD graph describes the `WebSite`, `WebPage`, and `VideoGame`, including both names, the official repository, free single-player browser gameplay, a real screenshot, and the Play destination. The visible overview explains the game's survival mechanics and political satire. It contains no invented reviews, ratings, or awards.

Implementation references: [Google site names](https://developers.google.com/search/docs/appearance/site-names), [Google canonical URL guidance](https://developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls), [Google sitemap guidance](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap), and [Schema.org VideoGame](https://schema.org/VideoGame).

Search rankings and indexing cannot be guaranteed by markup or Lighthouse. After an approved production release, verify the domain in Search Console, submit the sitemap, inspect Google's selected canonical, and verify the actual production URLs. Domain-level redirects and existing public profiles should consistently point to `https://dystrail.com/`; no account or hosting configuration was changed in this implementation.

## Validation

Lighthouse 13.4.1, isolated Chrome, local Python HTTP server, standard mobile throttling and desktop preset:

| Audit | Mobile | Desktop |
| --- | ---: | ---: |
| Performance | 99 | 100 |
| Accessibility | 100 | 100 |
| Best practices | 100 | 100 |
| SEO | 100 | 100 |
| First contentful paint | 0.8 s | 0.2 s |
| Largest contentful paint | 2.1 s | 0.8 s |
| Total blocking time | 0 ms | 0 ms |
| Cumulative layout shift | 0 | 0 |

These are local lab measurements, not production field data or a guarantee of future scores. The local server does not provide production caching or compression headers. Full Lighthouse HTML/JSON reports are in `/private/tmp/dystrail-website-review/`.

Additional checks:

- Visually inspected 320, 390, 768, and 1440-pixel layouts against the approved direction; no horizontal overflow. The first Play action is visible in each initial viewport.
- Keyboard traversal, visible focus outlines, and the skip link work. Automated accessibility/contrast audits pass.
- Tested 200% root text sizing at all four widths; corrected tablet content clipping.
- Reduced motion leaves no active animations. Content and documentation navigation work with JavaScript disabled.
- All homepage assets and both internal destination routes return HTTP 200 with appropriate content types. JSON-LD parses, identifiers are unique, image dimensions and alt attributes are present, and canonical/sitemap URLs agree.
- No third-party homepage requests, game bundle preloads, or homepage JavaScript errors. Fonts and optimized WebP imagery are served locally; the gameplay image is lazy-loaded.
- A fresh release game build succeeds on the isolated preview origin. Clicking Play opens `/play/` and its current character-selection entry point. Documentation is freshly built with mdBook.
- Existing saves on other origins are untouched. No game-state code, accounts, APIs, or backend services were added.

## Artwork and fonts

- Prairie: `dystrail-web/static/img/journey/open-heartland-prairie.png` → responsive WebP derivatives.
- Van: `dystrail-web/static/img/journey/van-crew.png` → responsive WebP derivatives preserving transparency.
- Actual gameplay: `docs/ux/review-2026-09-11/implementation/screenshots/travel-chromium.png` → responsive WebP and a 1200 × 630 JPEG social crop.
- Favicon: existing game favicon.
- Self-hosted Press Start 2P and Space Grotesk Latin WOFF2 fonts; their SIL Open Font Licenses are retained in `site/assets/fonts/`.

The original artwork and gameplay capture were not modified. Optimized derivatives and the social crop are website assets.

## Published release

Published September 11, 2026 at [dystrail.com](https://dystrail.com/) with the playable game at [/play/](https://dystrail.com/play/) and documentation at [/docs/](https://dystrail.com/docs/). [Deployment run](https://github.com/VannaDii/Dystrail/actions/runs/34642769554) succeeded for commit `b4a6bcd1bc5bd2d402e384c2f891f9ad72f28713` on `release/website-game-2026-09-11`. The separate default-branch history and continuing local game work were preserved.

The approved connection claim is now published after successful offline verification. On the live release, a fresh isolated Chrome profile completed setup, saved a journey, reached offline-ready status, reloaded with networking disabled, retained its saved crew, and loaded all 79 packaged assets from offline storage. Offline use follows the initial online load and successful asset preparation.

All 17 website files match the release bytes. The canonical, JSON-LD, sitemap, HTTPS/www redirects, game WASM content type, and documentation route passed production checks. Temporary permission for the release branch was explicitly approved, used for this deployment, and removed afterward; only the original `main` deployment permission remains. See `production-verification.json` and the live screenshots in this directory.
