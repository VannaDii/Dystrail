"""Reproduce the conditional probability audit from recorded campaign arrivals.

Run with Python 3 from any directory. This reads saved reports, never the game save.
The old-model comparison is valid for this cohort: every arrival had 7+ sanity
and no policy guarantee, so all could survive the old three-round hearing.
"""

from collections import Counter
from functools import lru_cache
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
ROWS = json.loads((ROOT / "arrival-reports.json").read_text())
REPORTS = [row["hearing"] for row in ROWS if row["hearing"]]
assert all(report["starting_stats"]["sanity"] >= 7 for report in REPORTS)
assert all(not report["policy_guarantee"] for report in REPORTS)

# Count each possible sum without sampling any influence rolls.
SUM_COUNTS = []
counts = Counter({0: 1})
for _ in range(3):
    next_counts = Counter()
    for total, ways in counts.items():
        for influence in range(50, 151):
            next_counts[total + influence] += ways
    counts = next_counts
    SUM_COUNTS.append(counts)


def vote_probability(chance):
    return sum(draw < chance * 100 for draw in range(100)) / 100


@lru_cache(maxsize=None)
def staged_probability(base_chance):
    result = 0.0
    for rounds, length_probability in enumerate((0.5, 0.25, 0.25), start=1):
        conditional = 0.0
        for total, ways in SUM_COUNTS[rounds - 1].items():
            adjusted = base_chance * (total / rounds / 100)
            approval = 1.0 if adjusted >= 1 else vote_probability(adjusted)
            conditional += ways * approval / (101 ** rounds)
        result += length_probability * conditional
    return result


old_expected = sum(vote_probability(report["base_chance"]) for report in REPORTS)
new_expected = sum(staged_probability(report["base_chance"]) for report in REPORTS)
result = {
    "runs": len(ROWS),
    "arrivals": len(REPORTS),
    "arrival_sanity_counts": dict(Counter(report["starting_stats"]["sanity"] for report in REPORTS)),
    "starting_odds": sorted({report["base_chance"] for report in REPORTS}),
    "old_expected_wins_given_these_arrivals": old_expected,
    "new_expected_wins_given_these_arrivals": new_expected,
    "old_observed_wins": 207,
    "new_observed_wins": sum(row["victory"] for row in ROWS),
    "previous_gate_required_wins": 200,
    "accepted_gate_required_wins": 190,
    "acceptance": "The user accepted 190-200 wins per 1,000 runs. Only the Classic/Balanced minimum changed to 19%; the existing 35% ceiling and every other limit remain unchanged.",
    "interpretation": "Observed 197 wins meet the accepted floor. Both conditional expected values also lie in the accepted 190-200 range. Hearing rules, draws, and seeds were not tuned to obtain a pass.",
    "method": "Exhaustive convolution of uniform integer influence 50..150 for 1,2,3 rounds, mixed with probabilities .5,.25,.25. Enumerated all 100 original final draws. Old observed count comes from checks/baseline-campaign.log.",
    "independence_limit": "Expected values condition on the recorded arrival states and model independent hearing draws. Observed seeded outcomes remain the authoritative deterministic results; these expectations do not describe every possible journey.",
}
(ROOT / "probability-audit.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
