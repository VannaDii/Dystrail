# Remaining campaign balance decision

The delivered controls let the player choose full days in camp, gather supplies, help crew members, buy any starting loadout, and earn cash or receipts from encounters. Those features remain available.

The existing acceptance code in `dystrail-tester/src/logic/playability.rs` requires **every** automated run to travel on at least **90%** of its recorded days. It also rejects Classic/Balanced samples if more than **50%** reach D.C. or more than **35%** win the hearing. The September 12 remeasurement of all 8,000 runs still fails those requirements after the varied local rewards and zero-health repair safeguard. Classic/Balanced reaches D.C. in 68.6% of runs and wins 42.0%. [Current measurements](remeasurement-2026-09-12.md).

Two distinct follow-ups are available:

1. **Preserve current gameplay and revise the campaign gate.** Keep the recorded 90% pacing floor and old success bands visible as balance comparisons. Make completion depend on actual invariants: actions charge their advertised costs, camp does not move the route, impossible choices do not execute, terminal outcomes stop further activity, state and RNG survive replay, and journeys finish without loops or invalid resources. Keep all existing raw results and report how the new distribution differs.
2. **Tune gameplay to the old success and pacing bands.** Keep the existing gates as hard failures and rebalance travel, recovery, encounters and the hearing toward those bands. This changes difficulty and requires another measured campaign. A pass would never be achieved by crediting miles while stationary, hiding failures, or reducing game features.

Recommendation: preserve the player-facing camp and reward behavior while revising obsolete balance gates explicitly. No acceptance threshold or gate has been changed yet.
