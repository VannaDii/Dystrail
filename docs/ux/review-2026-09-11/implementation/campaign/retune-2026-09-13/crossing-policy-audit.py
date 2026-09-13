"""Read-only crossing-roll analysis; not a campaign or gameplay test.

Reconstructs native rand 0.8.5 SmallRng's three crossing samples from the
repository's HMAC domain seed and local dependency source. It validates the
current policy against existing CSV telemetry before forecasting a bounded
policy change. Forecasts do not recompute clock, health, decisions or distance.
"""
import collections
import csv
import hashlib
import hmac
import json
from pathlib import Path
import struct

ROOT = Path('/tmp/dystrail-retune-2026-09-13')
MASK64 = (1 << 64) - 1
MASK32 = (1 << 32) - 1

def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]

def rotate(value, by, bits):
    return ((value << by) | (value >> (bits-by))) & ((1 << bits) - 1)

def samples(user_seed):
    seed = int.from_bytes(hmac.new(user_seed.to_bytes(8, 'little'), b'crossing', hashlib.sha256).digest()[:8], 'little')
    raw = b''
    for _ in range(8):
        seed = (seed * 6364136223846793005 + 11634580027462260723) & MASK64
        x = (((seed >> 18) ^ seed) >> 27) & MASK32
        rot = seed >> 59
        raw += rotate(x, (32-rot) % 32, 32).to_bytes(4, 'little')
    s = list(struct.unpack('<4Q', raw))
    for _ in range(3):
        value = (rotate((s[0]+s[3]) & MASK64, 23, 64) + s[0]) & MASK64
        t = (s[1] << 17) & MASK64
        s[2] ^= s[0]; s[3] ^= s[1]; s[1] ^= s[2]; s[0] ^= s[3]
        s[2] ^= t; s[3] = rotate(s[3], 45, 64)
        yield f32(((value >> 32) + 0.5) / (1 << 32))

def probabilities(policy, attempt):
    p, d, t = map(f32, policy[:3])
    total = f32(f32(p+d)+t)
    p,d,t = (f32(x/total) for x in (p,d,t))
    factor = f32(1/(1+float(f32(policy[5]))*attempt))
    p = f32(float(f32(policy[3]))*factor+p)
    t = max(f32(-float(f32(policy[4]))*factor+t), 0)
    total = f32(f32(p+d)+t)
    norm = f32(1/total)
    return [f32(x*norm) for x in (p,d,t)]

def outcomes(seed, policy, limit=3):
    result = []
    for i, draw in enumerate(samples(seed)):
        if i >= limit:
            break
        p,d,t = probabilities(policy,i)
        outcome = 'pass' if draw < p else 'detour' if draw < f32(p+d) else 'terminal'
        result.append(outcome)
        if i > 0 and outcome == 'terminal':
            break
    return result

def totals(outcome):
    return {
        'crossing_events':len(outcome),
        'crossing_bribe_attempts':len(outcome),
        'crossing_bribe_successes':outcome.count('pass'),
        'crossing_detours_taken':outcome.count('detour')+int(outcome[0]=='terminal'),
        'crossing_failures':int(outcome[-1]=='terminal' and len(outcome)>1),
    }

baseline = (.66,.24,.10,.192,.045,.38)
variants = {
    'current':baseline,
    'detour_008_only':(.66,.08,.10,.192,.045,.38),
    'terminal_020_only':(.66,.24,.20,.192,.045,.38),
    'bribe_pass_022_only':(.66,.24,.10,.22,.045,.38),
    'shift_detour_to_terminal':(.66,.14,.20,.192,.045,.38),
    'redistribute_070_010_020':(.70,.10,.20,.192,.045,.38),
    'redistribute_070_008_022':(.70,.08,.22,.192,.045,.38),
}
pace_rows = [row for row in csv.DictReader((ROOT/'pace-results.csv').open()) if row['scenario']=='Classic - Balanced']
parity_errors = []
for row in pace_rows:
    predicted = totals(outcomes(int(row['seed_value']), baseline))
    actual = {k:int(row[k]) for k in predicted}
    if predicted != actual:
        parity_errors.append({'seed':row['seed_value'],'predicted':predicted,'actual':actual})
assert not parity_errors, parity_errors[:5]

report = {'scope':'Crossing-only roll replay. No full-game or timing forecast. First checkpoint stays protected.',
          'native_baseline_250_seed_telemetry_parity':True,
          'variants':{}}
for name, policy in variants.items():
    counter = collections.Counter()
    terminal_indices = collections.Counter()
    for seed in range(1337,1587):
        result = outcomes(seed,policy)
        counter.update(totals(result))
        if result[-1]=='terminal' and len(result)>1:
            terminal_indices[len(result)-1] += 1
    report['variants'][name] = {
        'raw_policy':dict(zip(['pass','detour','terminal','bribe_pass_bonus','bribe_terminal_penalty','bribe_diminishing'],policy)),
        'effective_bribed_probabilities_by_attempt':[probabilities(policy,i) for i in range(3)],
        'crossing_only_totals':dict(counter),
        'bribe_success':counter['crossing_bribe_successes']/counter['crossing_bribe_attempts'],
        'terminal_rate':counter['crossing_failures']/counter['crossing_events'],
        'terminal_by_crossing_index':dict(terminal_indices),
    }
(ROOT/'crossing-policy-audit.json').write_text(json.dumps(report,indent=2)+'\n')
print('Baseline: exact 250-seed per-run parity for event count, all bribes, successful bribes, detours, terminal failures.')
for name,d in report['variants'].items():
    print(name, 'bribe',round(d['bribe_success'],4), 'terminal',round(d['terminal_rate'],4),
          'terminal_indices',d['terminal_by_crossing_index'], 'totals',d['crossing_only_totals'])
