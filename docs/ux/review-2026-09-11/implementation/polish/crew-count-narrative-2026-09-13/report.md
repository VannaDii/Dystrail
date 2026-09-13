# Crew-neutral encounter copy — complete release scope

Updated 2026-09-13. Candidate revision `1c145d89fef4fbab75a9`.

This narrative-only change removes fixed player-crew headcounts while preserving the writers’ actions and jokes. The final scope is **421 string leaves across 21 JSON files**: 20 common fields in 15 encounters, each present in canonical game data and all 20 existing locale bundles (420 leaves), plus one Spanish-only correction.

## Exact scope

| Encounter | Fields |
|---|---|
| `beltway_briefing` | `choice_2`, `log_2` |
| `classic_union_blockade` | `desc`, `choice_0`, `log_0` |
| `classic_mail_drop` | `log_0` |
| `classic_mutual_aid_dispatch` | `log_0` |
| `classic_neighborhood_watch` | `log_2` |
| `classic_press_pool_qna` | `desc` |
| `deep_circuit_breaker` | `log_0` |
| `fundraiser_detour` | `log_0` |
| `deep_beltway_fastpass` | `log_1` |
| `overnight_briefing` | `log_2`, `name` |
| `sat_straw_reserve` | `log_2` |
| `sat_forecast_corrected` | `log_2` |
| `sat_alternator_tariff` | `desc` |
| `sat_hydration_pressure` | `desc`, `log_2` |
| `sat_bibliography_emergency` | `log_2` |

The title is now “Five Accomplishments, One Exhausted Crew.” The separate Spanish correction changes `sat_receipt_museum.log_2` from “Ninguno de los seis…” to “Ninguno de ustedes…”, matching the canonical “None of you…” wording. Italian, Spanish and Arabic retain their existing translations; the other 17 bundles retain their existing English fallback convention.

## Preservation

The reproducible deep comparison verifies exact key sets and unchanged values outside the 421 permitted leaves. IDs, choice order/count, effect amounts and operations, probabilities, timing, sources, placeholders and paragraph breaks are unchanged. Five accomplishments/reasons, one van/two deliveries, nine-dollar pricing, and phone/cable/seat object or capacity counts remain intact. The canonical English and English locale values agree exactly.

The frozen release was copied from `/tmp/dystrail-visual-refine/unit5-source` and overlaid with only the 21 approved JSON files. Of 634 source files, the remaining 613 are byte-identical. Published camp rest sanity remains **+4**. No balance experiment was included.

Frozen source tree manifest SHA-256: `76c82b2e59b7c6210a9f3382e92e275ee5792e8108a700987441b7fa17516038`.

## Compiled and browser verification

Trunk release compilation exited 0. All 119 asset hashes and sizes (109,953,920 bytes) match the generated offline manifest; the service-worker manifest matches exactly.

Chrome with Node 24.19.0 resolved 64 encounters across English, Italian, Spanish and Arabic with three active crew members. Each scene and outcome was checked at 1440×1000 and 393×852: 128 scene inspections and 128 aftermath inspections. Imported encounter objects came from the old published source, demonstrating that revised locale copy is used for persisted saves.

The browser checks assert the complete localized body, all choice labels/counts, localized title inclusion, selected-choice availability, expected outcome, unchanged crew persona/status pairs, an active speaker, a cleared encounter after resolution, and no horizontal overflow. All four runs and final language-preserving reloads ran offline after caching. No page errors occurred.

Numeric elapsed time and resource deltas were not independently asserted by this browser script. Their source fields, engine code and configuration are unchanged byte-for-byte. `assertion-coverage.json` records the precise scope and this limit.

Clean user-review screenshots from the published preview are [Classic phone](en-overnight_briefing-phone-clean.png) and [Deep phone](en-beltway_briefing-phone-clean.png). Both were captured after the temporary import notification cleared naturally; its DOM container is intentionally retained empty and hidden. The prior screenshot states remain unchanged. `clean-phone-results.json` verifies the live revision, exact body/title, no horizontal overflow, and no page errors. Other representative phone screenshots are `ar-beltway_briefing-phone.png` (RTL Deep) and `es-museum-outcome-phone.png` (Spanish-only correction). Full local evidence includes 28 screenshots.

## Evidence and publication boundary

`scope.json`, `changes.json`, `diff.patch`, `validate.py`, and `validation.json` describe and reproduce the 421-leaf change. `release-source-verification.json` proves the 21-file overlay. `artifact-verification.json`, `browser-results.json`, and `assertion-coverage.json` retain compiled/browser evidence.

The earlier 105-leaf report is retained as `report-initial-105-leaves.md`. Its initial outside-scope list is historical: all approved crew references in that list are now included. It is not an outstanding-work list.

Preview publication completed at `http://127.0.0.1:8180/play/`, revision `1c145d89fef4fbab75a9`. All 119 live assets and the service worker match the verified candidate. Save storage was untouched. The receipt is `preview-publication.json` (also retained as the sibling `preview-1c145d89fef4fbab75a9.json`). Production remained unchanged at the end of this preview verification phase. The subsequently authorized production release is recorded in [the production deployment report](../narrative-1c145d89fef4fbab75a9/DEPLOYMENT.md).

## Frozen file SHA-256

- `dystrail-web/i18n/ar.json`: `4209618b8152c0e54e893ad347e689232122b8bcc1a92283604107b552b8e5b5` (20 string leaves)
- `dystrail-web/i18n/bn.json`: `0377ba14563c9ac862b2b7d62be725ded24c60b20655726428580d7634b95093` (20 string leaves)
- `dystrail-web/i18n/de.json`: `c1ffa2a1c7521ffe1e96fe298879b414cf8ae3a839599780ebc09c8d67e0d38a` (20 string leaves)
- `dystrail-web/i18n/en.json`: `0dc8897f620b3baad32cfa9564bab61b3c7bc27f4f7f7590ef3dff1738e1d553` (20 string leaves)
- `dystrail-web/i18n/es.json`: `f98a4b90895109d27b724c7872b04e0b04950616aab5bf6bdb7aeb0508b80ada` (21 string leaves)
- `dystrail-web/i18n/fr.json`: `7a42b577aab851dd8847e0dd09959c097b4a2488f80cd7382fd8d4c961e939fe` (20 string leaves)
- `dystrail-web/i18n/hi.json`: `b04375d7bd70b022e682fd5755eb50b05acf1f9c9321d7dcf78ae772654eb49a` (20 string leaves)
- `dystrail-web/i18n/id.json`: `5badd07fc2e939ceb53cae1467ad787f0a2e82640e190178f251fe95bc1fd7a8` (20 string leaves)
- `dystrail-web/i18n/it.json`: `00ccb5ed3f644fdbcbcc77ff8590ed3282aea1bcf2f75b797e74c4e434bb6eab` (20 string leaves)
- `dystrail-web/i18n/ja.json`: `85ca4c45454bbb3f1b33ea27b484af04e6bff2c82304d14c114d59724bc8cb8d` (20 string leaves)
- `dystrail-web/i18n/jv.json`: `2783356c981a656d6f3c6a9dd52287df7c53ff5bb2bd4efa80ee749582558a25` (20 string leaves)
- `dystrail-web/i18n/ko.json`: `18806f6de9dbc60f5e4720f6ebb694c6d0922128a5820cc4fa59ec3a73a6972a` (20 string leaves)
- `dystrail-web/i18n/mr.json`: `6f348b65e6c41c84ad15fbed91de6d530bc73e914138fee8b0999489c8d7b933` (20 string leaves)
- `dystrail-web/i18n/pa.json`: `936fa952fb331b8746065a2a9e3ffd18e1e24c85416aaef80831a044ce04300e` (20 string leaves)
- `dystrail-web/i18n/pt.json`: `446a394c2fa804a3e3f335b3ae168b5669c457f01b769504d552ba8d61dad424` (20 string leaves)
- `dystrail-web/i18n/ru.json`: `61affc6c0ef32cc45e92b54edcde7f913021605e8bbb4a7d1309adfbdcb3794a` (20 string leaves)
- `dystrail-web/i18n/ta.json`: `30d0e5a3ec3979fdc4740e3bdcb255f27a3150dcc252ae022d3b91eff3049da1` (20 string leaves)
- `dystrail-web/i18n/te.json`: `cab2a5155a7b01e81677dd50a72a720b833b8fc9e72f7b747348c2055d773a0e` (20 string leaves)
- `dystrail-web/i18n/tr.json`: `8702cd07043180a92253d1dee8341dda0b6529539c70d0cf6402f7382db3b21a` (20 string leaves)
- `dystrail-web/i18n/zh.json`: `0003b9498faa1e408a7cd21782a386b242428226874d53d3e8b7450921dabb7d` (20 string leaves)
- `dystrail-web/static/assets/data/game.json`: `6f57ab062390d5d3145ff795dea9c643375c82f91925e2abb0e36a1747e2fa8c` (20 string leaves)
