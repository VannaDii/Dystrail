# Encounter arrival preview

Revision: `579bc5ab64918159ae27`. One shared action now resolves the encounter, its thirty-minute cost, reached crossing or town, and new-day costs. Crossing delays remain additional real time; the outcome explains passage fees with existing localized copy. Browser and tester use the same engine action. No strategies, acceptance assertions, effect values or balance settings were changed.

211 engine library tests, formatting, strict native/Wasm lint and release build pass. Existing tester suite: 23 pass; the two retained deterministic fingerprint expectations remain open with the ongoing balance work. Their assertions were not changed.

Four browser cases pass (both themes, morning and overnight arrival): exact milestone, one minute of driving for the final half-mile, thirty encounter minutes plus thirty crossing minutes, correct existing per-mode fee, one crossing resolution, immediate new-day costs, explanation text, unchanged pause and offline reload, no runtime errors. Engine cases additionally cover detours, terminal crossings, town arrival, unaffordable/repeated choices and pending breakdowns.

The initial new test fixture had zero configured daily food cost, and the first browser fixture assumed imported derived crossing policy was retained. Those fixture assumptions were corrected; existing test expectations and product costs were preserved.

Production remains unchanged. Experimental balance settings remain isolated.
