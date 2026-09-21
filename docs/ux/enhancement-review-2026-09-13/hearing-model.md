**Dystrail: staged hearing model**

Implementation update, September 13: the approved hearing is now built locally. See the [implemented review and accepted campaign balance](implementation/hearing/README.md). The material below records the design review that preceded implementation.

Planning proposal, September 13, 2026. This follows the user's decision to retain a random final vote influenced by the existing stat-based odds. It replaces the previous deterministic-victory recommendation. No gameplay changes have been implemented.

**Direction and provisional parameters**

Keep the existing starting-odds calculation. Always hold an opening round, randomly determine whether a second and then a third round occurs, and charge 2 sanity for every round actually rolled. If sanity reaches zero, the hearing fails. Otherwise average the round results to modify the starting odds. Once the hearing closes, adjusted odds of at least 100% produce an automatic victory; lower odds go to the final random vote.

The following values are the working baseline carried into the accepted implementation goal and storyboard. Campaign tuning remains subject to measured testing:

- Round influence values: uniformly distributed integers from **50 through 150**, interpreted as percentages. A 100 result is neutral; 120 means a 1.20 multiplier. This allows both help and harm, as represented in the accepted goal and storyboard.
- Continuation: **50% after round 1, 50% after round 2**, independent of performance and stats for the first model. The third round always ends the hearing if the crew survives.
- Average **only rounds that occurred**. A one-round hearing has one value in its average. Continuation rolls and the final vote are not part of that average.
- Retain the existing final draw convention for this candidate: a uniformly distributed integer from 0 through 99, compared with the unrounded adjusted percentage.

Using a normal 0–100 roll directly as a percentage multiplier would have an average near 0.5 and could never improve starting odds. Dividing a one-round result by three would impose another unrelated penalty. Both would conflict with the intended role of the new rounds.

**Resolution sequence**

1. Capture starting odds from the existing stat/score formula at hearing entry, after any existing applicable policy preparation. Freeze that value for this hearing. Do not keep recomputing it from time spent displaying rounds.
2. Roll the opening round and apply its 2-sanity cost exactly once. Record the performance and before/after sanity together. At zero sanity, stop with exhaustion; a strong influence result cannot rescue that failure.
3. If the crew survives, roll whether the committee calls a second round. This continuation check costs no sanity. If it closes the hearing, proceed to the adjusted-odds calculation.
4. If called, resolve round 2 and its 2-sanity cost. On survival, roll whether there is a third round. Again, the continuation check itself costs nothing.
5. If called, resolve round 3 and its 2-sanity cost. There is no fourth round.
6. After the last actual round, if sanity remains positive, calculate the average influence and adjusted odds.
7. If adjusted odds are at least 100%, declare victory without a final vote draw. Otherwise make the final vote draw and compare it with those odds. That final draw costs no additional sanity.

Do not award automatic victory while another round can still be called: a later result can lower the average, and a later sanity cost can end the hearing.

For `n` rounds, with percentage influence values `r1 ... rn`:

```text
average influence = (r1 + ... + rn) / n
adjusted odds (%) = starting odds (%) × average influence / 100
```

The existing 25%–88% bounds remain part of the base formula where applicable. Do not reapply them after the new multiplier: an 88% final cap would make the requested automatic victory impossible. The new effective probability is bounded by 0% and 100%. Check the unrounded value for automatic victory, so a display rounded to “100%” does not falsely promise certainty.

The current internal Deep/Aggressive policy can set base odds to 100%. The accepted implementation goal preserves that explicit guarantee if the crew survives; do not inadvertently turn it into a lower chance through a weak influence roll.

| Starting odds | Actual round influence values | Average | Adjusted odds | Sanity cost | Outcome path |
|---|---|---|---|---|---|
| 55% | 120 | 120% | 66% | 2 | Final vote at 66% |
| 55% | 140, 100 | 120% | 66% | 4 | Final vote at 66% |
| 80% | 140, 130, 120 | 130% | 104% | 6 | Automatic victory if sanity remains positive |
| 55% | 60, 100 | 80% | 44% | 4 | Final vote at 44% |

Under this candidate range, a 55% base can reach at most 82.5%, so it cannot earn automatic victory through influence alone. A starting chance of at least two-thirds can reach 100% with sufficiently strong rounds. Wider ranges can permit automatic wins from weaker starts but introduce larger setbacks and more variance. Choose that tradeoff deliberately rather than changing the range solely to produce a dramatic example.

**Balance implications checked arithmetically**

With independent 50% continuation checks, planned hearing lengths are 50% one round, 25% two rounds, and 25% three rounds. A crew able to survive every possible length experiences an average of 1.75 rounds and spends 3.5 sanity. The existing hearing always attempts three rounds and charges up to 6 sanity.

| Entry sanity, after policy preparation | Chance of surviving until the vote/automatic result |
|---|---|
| 1–2 | 0% |
| 3–4 | 50% |
| 5–6 | 75% |
| 7 or more | 100% |

This changes the value of resting before the hearing. A crew that cannot survive three rounds can now succeed if the committee closes earlier. The current preview, which simulates all three sanity deductions and rejects exhausted cases, must be replaced. Show starting vote odds and the separate risk of exhaustion, or calculate an overall forecast from this same model; do not label the unmodified starting odds as the whole chance of winning.

An average multiplier of 1 does not preserve every original win rate exactly. Upside beyond 100% is clipped while downside still counts, which reduces the average result at strong starting odds. More rounds narrow the distribution around neutral, but cost more sanity; they are not automatically a reward. The current integer-percent final draw also rounds probability upward in sub-percent cases.

The following results were calculated by enumerating all influence-sum distributions for one, two, and three rounds, mixing the stated hearing lengths, and counting successful integer final draws. They assume at least 7 entry sanity and exclude the special base-100 policy. These are checks of the isolated proposed formula, not observed campaign balance or production results.

| Starting odds | Modeled overall win rate | Hearings ending in automatic victory |
|---|---|---|
| 25% | 25.41% | 0% |
| 55% | 55.48% | 0% |
| 75% | 74.72% | 10.44% |
| 80% | 78.75% | 17.97% |
| 88% | 84.21% | 30.45% |

Higher starting odds still help for the same influence results and stamina. However, the reduced sanity burden and the clipping effect move balance in different directions. Tune continuation chances, influence range/distribution, and any boost-only alternative against complete runs before selecting final values. Do not describe this candidate as preserving the existing win rates exactly.

**Presentation and player engagement**

The [arrival-to-verdict storyboard](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/hearing-storyboard.md) shows the intended scenes, actions, feelings, and alternative endings.

Present the sequence as an opening argument, a possible follow-up, and a possible final challenge. Each round reveals a satirical reaction, influence result, and its sanity cost. The committee either calls another round or proceeds to the vote. Show a provisional adjusted chance while another round is possible; reserve a confirmed probability and automatic-win declaration for the closed hearing.

Allow the player to advance the round reveals, with automatic playback/skip and reduced-motion equivalents. Do not require an extra click for every hidden continuation check. Use distinct closing beats for exhaustion, automatic victory, a passed final vote, and a failed final vote. Additional required content is three round contexts, weak/neutral/strong reaction variants, two continuation-or-close announcements, and the four closing states; coordinate these with the existing hearing/ending brief.

This gives the player more suspense and control over pacing. Choosing when to reveal a stored result does not provide tactical agency. Decisions that trade resources, influence, or sanity between rounds would be an optional later design addition; they are not assumed by this proposal.

**Persistence, replay, and the scorecard**

For the baseline without tactical decisions, resolve and persist the hearing report once, then reveal it progressively. Store the base-odds snapshot, actual performance rolls, continuation results, per-round sanity, final average, adjusted odds, any final draw, and outcome. Preserve the existing dedicated hearing random stream and version the rules because the new draw sequence changes same-seed results. Reload, skip, fast mode, and double-clicking must not generate new rolls or charge sanity again. All visible stats should follow the presented report stage rather than revealing later deductions early.

The scorecard should explain starting odds, the actual hearing influence, adjusted odds, and the result. A “what would have won” stat comparison can hold the completed hearing's rolls fixed and find the smallest valid stat changes that would beat that particular final draw. Label that comparison as specific to this hearing, not a guarantee for another playthrough. Some failed rolls cannot be overcome by any legal stat-only change within the base-odds cap. Exhaustion has no actual final vote draw to reconstruct; report the sanity shortfall and avoid inventing unseen outcomes. A display-score target remains distinct from the chance of winning.

Implementation checks must cover one/two/three rounds, sanity at 2/3/4/5/6/7, actual-round averaging, cost-free continuation checks, clipped and sub-100 probabilities, the existing policy guarantee, no final draw after automatic victory, no early victory before the last round, preview accuracy, saved reveals, old rules, and scorecard counterfactual limits.

Source anchors: [current hearing resolution and odds](/Users/vanna/Source/Dystrail/dystrail-game/src/boss.rs:100), [hearing configuration](/Users/vanna/Source/Dystrail/dystrail-web/static/assets/data/boss.json), [current integer-percent draw](/Users/vanna/Source/Dystrail/dystrail-game/src/state.rs:5233). See the [integrated implementation plan](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/plan.md).
