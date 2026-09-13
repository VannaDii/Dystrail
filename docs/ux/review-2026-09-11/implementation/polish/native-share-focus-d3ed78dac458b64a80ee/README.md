# Final sharing focus fix and goal closure — September 13, 2026

Preview port 8180 and [production](https://dystrail.com/play/) serve **d3ed78dac458b64a80ee**. [Commit 99ae520b19f1ef8eb259d2211229fe954e4d3803](https://github.com/VannaDii/Dystrail/commit/99ae520b19f1ef8eb259d2211229fe954e4d3803) was published through [successful Pages run 34782506065](https://github.com/VannaDii/Dystrail/actions/runs/34782506065).

## Change

Canceling Safari's system share sheet could leave keyboard focus outside the game's sharing dialog. The share button now receives focus after the sharing operation finishes and the button is enabled again. The edited post and saved journey remain unchanged. This is the only production source change from 33dd2b05a6658d330912; image composition, CSS, fonts, content, game rules and sharing payload are preserved. [Exact source delta](source.patch).

## Completed verification

- The user confirms successful installation and offline reopening on a physical phone. This is direct user evidence, not an independently observed device test.
- The actual Safari/macOS share sheet opened with the generated PNG on the preceding artifact. Canceling returned to the unchanged edited draft with sharing re-enabled; it exposed the focus defect. No recipient was selected. [Native observation and its limits](native-sheet-observation.json).
- The regression fails at missing focus on the preceding build. All **12** focused sharing cases pass on the fix, covering desktop and phone layouts, edited payloads, cancellation/failure/completion, keyboard focus, export, unsupported capabilities and rendering failures.
- **Native Safari 26.6.2** passes **eight** focused groups across Classic and Deep: controlled canceled/failed/completed sharing promises, restored focus, Tab/Shift+Tab, Escape, unchanged draft and unchanged complete saved game. This is a real Safari focus test, not a new assertion that the OS sheet appeared. The first harness attempt hit an offscreen button; its failure is retained. Scrolling the test control into view resolves that test setup issue.
- **79** frontend unit tests, strict native and Wasm lint, release build and the unmodified game-client smoke pass. Its screenshot was inspected during verification.
- Updating an existing disposable installation from the preceding artifact preserves the complete save. Offline reload and fully closed-browser offline restart pass. All **119** assets match their cache hashes, with **68** prepared images and **seven** loaded fonts.
- All **207** live file hashes and MIME types and **six** canonical routes pass. **59** non-game site files and **31** older hashed assets are preserved. The exact temporary release permission is removed and main-only deployment policy restored. Preview was updated immediately after the focused fix passed.

The accepted hourly balance and final writer copy remain unchanged. The retained **2,000** campaign runs, **364** native tests, **two** Wasm tests and **408** browser cases apply to that prior accepted baseline; they were not repeated for this focus-only change. The existing native gameplay, offline and ending assertion groups remain attributed to their recorded builds.

## Completion boundary

All implementation, validation and delivery work in the original goal and later corrections is complete. Actual OS sharing and cancellation were observed before this focus-only fix; the resulting focus behavior was verified afterward through controlled completions in real Safari. A final manual OS-sheet replay was unavailable and is not claimed. This proportional verification covers the changed behavior without repeating the full campaign or visual review. Recipient delivery was not attempted; phone installation/offline success is user-reported.

[Consolidated completed requirements](../../goal-outstanding-2026-09-12.md), [full player feedback](../../feedback.md), and [prior visual release](../share-hud-33dd2b05a6658d330912/README.md) retain the complete scope and historical evidence.
