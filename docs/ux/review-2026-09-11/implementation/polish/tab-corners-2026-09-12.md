# Rounded tab corners

Published preview revision: `0f97e366c7a71602378d`.

Rounded tabs use a separate inset underline, removing the partial-border corner artifacts. Only the verified stylesheet changed; Wasm and game data are byte-identical to preview `7050e40f35ec177a2368`.

Checks: 6 desktop/phone entry-layout and navigation cases passed with exit 0; native Safari inventory/navigation suite passed 14 groups with exit 0. Desktop, phone and native Safari screenshots were inspected. The unmodified develop-web-game client produced two screenshots; the final rendered character screen was inspected. All 100 served assets and the service worker match the manifest after atomic preview publication. Save storage untouched.

Evidence: `/tmp/dystrail-tab-corners-browser.log`, `/tmp/dystrail-tab-corners-safari/validation.json`, `/tmp/dystrail-tab-corners-game-client/shot-1.png`, and `preview-0f97e366c7a71602378d.json`.
