"""Reject expired, broadened or stale maintenance-advisory exceptions."""
from datetime import date
import json
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def validate(today):
    exceptions = json.loads((ROOT / 'docs/security/advisory-exceptions.json').read_text())
    ids = {entry['id'] for entry in exceptions}
    assert len(ids) == len(exceptions), 'Duplicate advisory exception'
    for config in ['.cargo/audit.toml', 'deny.toml']:
        allowed = tomllib.loads((ROOT / config).read_text())['advisories']['ignore']
        assert set(allowed) == ids, f'Untracked advisory exception in {config}'
    packages = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    for entry in exceptions:
        assert today < date.fromisoformat(entry['expires']), f"Expired exception: {entry['id']}"
        versions = {p['version'] for p in packages if p['name'] == entry['package']}
        assert versions == set(entry['versions']), f"Dependency scope changed: {entry['id']}"
        assert (ROOT / entry['issue'].split('#')[0]).is_file(), f"Missing issue: {entry['id']}"
        print(f"Temporary maintenance exception: {entry['id']} / {entry['package']} "
              f"{', '.join(entry['versions'])}; expires {entry['expires']}; {entry['issue']}")


if __name__ == '__main__':
    validate(date.today())
