# Floating language menu and reliable entry URLs

Language choices now open outside the menu frame without expanding it or adding a second menu scrollbar. The list stays anchored to its button, fits the visible viewport, and opens above the button when space below is limited. All 20 languages remain reachable. Keyboard navigation, selection persistence, Arabic RTL, focus restoration, and separate Escape dismissal of the list and menu are preserved.

Opening `/play` now changes the address to `/play/` before offline preparation. The bare path was outside the service worker's `/play/` scope, causing preparation to stall on hosts that served it directly. The correction preserves query strings, fragments, browser history behavior, and saved journeys. Production's existing HTTP redirect is retained.

## Source and preview

- Language fix: `4a2e43acc3877b42e5ec1ee4daf770f576a95ade`.
- Entry URL fix: `c55a6434a9b09ed39a6ae9d7e21a0d521d939341`.
- Final test synchronization: `10f50ecef87e5ababf92cc0334ba623f94861aef`.
- Chevron spacing fix: `d7b22770302a2149392a5d34aa981b1f6141087b`.
- Source branch: `release/complete-game-source-2026-09-13`.
- Final CI: [34798552993](https://github.com/VannaDii/Dystrail/actions/runs/34798552993).
- Preview: `http://127.0.0.1:8180/play/`, revision `4a35a2024033bd2ce615`. Each implementation unit was refreshed after verification.

No gameplay rules, writer content, saved fields, or other tasks' drafts changed in this follow-up. The user's separately requested chevron spacing correction is included. The final CI and production verification are complete.

## Verification

Local checks passed: formatting, strict Wasm lint, 79 frontend tests plus three integration tests, and the release build. Eight language/accessibility browser cases cover desktop and phone layouts in both modes, unchanged menu geometry, all language choices, short and enlarged viewports, Arabic, persistence, keyboard focus, and dismissal. Screenshots and the original game-client smoke output were opened for visual review.

The missing-slash regression failed on the prior build as expected. After correction, 12 desktop/phone entry, saved/offline, loader-integrity, and language cases passed. A separate check passed on the actual port 8180 preview, and the original game client opened the formerly broken address successfully. The exact combined CI artifact also preserved the complete saved journey through update, offline reload, and closed-browser offline restart, with 119 cached assets, 68 images, and seven fonts ready.

The chevron hotfix was published separately as `5722f8b0b36b27c815902359a54a08e77e7c6589`, revision `d75df50dab2d09572dcf`. This combined release starts from that verified production baseline and preserves its hashed stylesheet. The complete save/update/offline verification was repeated from that baseline to the final combined artifact. Desktop Classic and phone Deep-mode preview checks confirm the bare address loads, the chevron retains its 13.8 px inset, the list floats outside the unchanged menu, and the last of all 20 languages remains reachable. Both final screenshots were visually reviewed.

## Test corrections and limits

The initial language test sampled layout before its resize observer completed and used an outside-click target covered by the upward-opening list. It now retries the same geometry assertions within the original five-second limit and clicks a verified outside point. Existing expectations were retained.

The first combined CI run, 34796470219, passed 421 of 422 browser cases and all gameplay gates. Its remaining retry check asserted before asset preparation finished; the failure snapshot contained the loaded game. The final test waits for the retry document and the existing launch-ready promise before its original visibility assertion. An exact normalized comparison confirms all assertions and timeouts are unchanged. Eight targeted desktop/phone retry and entry cases pass against the frozen preview. This test correction changes no product files. Its rebuilt JavaScript and Wasm bytes differed, so saved-install verification was repeated on the exact final CI artifact rather than reused. That final verification passed.

The follow-up run 34797629882 passed 13 of 14 jobs before the chevron source push automatically cancelled its last browser batch. The combined source run 34798552993 supersedes it; cancellation is not recorded as a pass.

An optional cached WebKit run could not start because its protocol did not match the installed runner. A matching isolated download stalled during extraction and was stopped. No WebKit or native Safari result is claimed for this follow-up; the user's Safari session and saved journey were not used for testing.

## Published result

All 14 source CI jobs passed, including the complete browser/accessibility suite, campaign QA, native/Wasm tests, translation coverage, strict lint, security checks, and release build.

- Production: https://dystrail.com/play/
- Revision: `4a35a2024033bd2ce615`; preview and production match.
- Source: `d7b22770302a2149392a5d34aa981b1f6141087b` on `release/complete-game-source-2026-09-13`.
- Artifact: `22aaa3aa4f12a45127fc5a5583646e492989fe72` on `release/source-handoff-publication-2026-09-13`.
- Deployment: [34799527294](https://github.com/VannaDii/Dystrail/actions/runs/34799527294).
- Live verification: all 218 file hashes and response types match; all 10 canonical URL cases pass, including bare `/play`, its query string, HTTP, and www redirects.
- Preserved: 59 website files, 39 older hashed assets, and the complete saved journey through update, offline reload, and closed-browser offline restart. All 119 cached assets, 68 images, and seven fonts are ready.
- Fresh production browsers: desktop Classic and phone Deep mode both load from the bare URL, preserve its query/fragment, retain chevron padding, expose all 20 languages in the floating list, and report no page errors.
- The temporary deployment permission was removed; the original main-only policy is restored. The port 8180 preview remains running; this task's temporary servers on 51804 and 51805 were closed.

Other tasks' content and hearing/storyboard drafts remain outside this release. No additional implementation or publication work remains for these fixes.
