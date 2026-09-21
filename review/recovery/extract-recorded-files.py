"""Recover recorded file additions/patches as evidence. Never execute recorded code."""
import ast
import hashlib
import json
import re
from pathlib import Path

SESSION = Path('/Users/vanna/.codex/sessions/2026/09/14/rollout-2026-09-14T10-29-39-01a0a0f7-a4be-7852-a652-0b497de9fbea.jsonl')
ROOT = Path(__file__).resolve().parent
PREFIX = '/private/tmp/dystrail-art-satire/'
state, events, gaps, literals = {}, {}, [], []

def patch(text, diff):
    lines = text.splitlines(keepends=True)
    chunks = re.split(r'(?m)^@@[^\n]*\n', diff)[1:]
    for chunk in chunks:
        old, new = [], []
        for line in chunk.splitlines(keepends=True):
            if line.startswith((' ', '-')): old.append(line[1:])
            if line.startswith((' ', '+')): new.append(line[1:])
        matches = [i for i in range(len(lines)-len(old)+1) if lines[i:i+len(old)] == old]
        if len(matches) != 1: raise ValueError('Patch context is absent or ambiguous')
        i = matches[0]
        lines[i:i+len(old)] = new
    return ''.join(lines)

with SESSION.open() as stream:
    for number, raw in enumerate(stream, 1):
        if number > 19400: break  # fixed historical boundary, before recovery work
        if len(raw) > 2_000_000: continue
        record = json.loads(raw)
        item = record.get('payload', {}).get('item', {})
        if item.get('type') == 'FileChange':
            for absolute, change in item.get('changes', {}).items():
                if not absolute.startswith(PREFIX): continue
                relative = absolute[len(PREFIX):]
                if '..' in Path(relative).parts: continue
                kind = change.get('type')
                try:
                    if kind == 'add': state[relative] = change['content']
                    elif kind == 'update' and relative in state:
                        state[relative] = patch(state[relative], change['unified_diff'])
                    else: raise ValueError('No recovered full-file base')
                    events.setdefault(relative, []).append(number)
                except ValueError as exc:
                    gaps.append({'path': relative, 'record': number, 'reason': str(exc)})
        if item.get('type') == 'CommandExecution':
            command = '\n'.join(item.get('command', []))
            if PREFIX.rstrip('/') not in str(item.get('cwd', '')): continue
            for found in re.finditer(r"cat\s*>\s*([^\s]+)\s*<<['\"]?([A-Za-z_][A-Za-z0-9_]*)['\"]?\s*\n", command):
                destination, delimiter = found.groups()
                tail = command[found.end():]
                end = re.search(r'(?m)^' + re.escape(delimiter) + r'$' , tail)
                if not end: continue
                destination = destination.strip("'\"")
                if destination.startswith(PREFIX): relative = destination[len(PREFIX):]
                elif destination.startswith('/') or '..' in Path(destination).parts: continue
                else:
                    cwd = str(item.get('cwd', '')).removeprefix('file://').rstrip('/')
                    sub = cwd[len(PREFIX):] if cwd.startswith(PREFIX) else ''
                    relative = str(Path(sub)/destination)
                state[relative] = tail[:end.start()]
                events.setdefault(relative, []).append(number)
            for code in re.findall(r"<<\s*['\"]?PY['\"]?\s*\n(.*?)\nPY(?:\n|$)", command, re.S):
                try: tree = ast.parse(code)
                except SyntaxError: continue
                for node in ast.walk(tree):
                    if isinstance(node, (ast.List, ast.Dict, ast.Tuple)):
                        try: value = ast.literal_eval(node)
                        except (ValueError, TypeError, SyntaxError): continue
                        encoded = json.dumps(value, ensure_ascii=False)
                        if re.search(r'[0-9a-f]{8}-[0-9a-f-]{27}', encoded):
                            literals.append({'record': number, 'value': value, 'recorded_command': command})

manifest = []
for relative, text in sorted(state.items()):
    target = ROOT/'recorded-files'/relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text)
    manifest.append({'path': relative, 'sha256': hashlib.sha256(text.encode()).hexdigest(),
                     'records': events[relative], 'status': 'historical_candidate_not_current_source',
                     'failed_patches': [g for g in gaps if g['path'] == relative]})
(ROOT/'recorded-file-index.json').write_text(json.dumps({'files': manifest, 'gaps': gaps}, indent=2)+'\n')
(ROOT/'recorded-asset-literals.json').write_text(json.dumps(literals, indent=2)+'\n')
print(json.dumps({'recovered_file_candidates': len(manifest), 'applied_file_events':sum(len(x['records']) for x in manifest), 'unapplied_events':len(gaps),'asset_literal_records':len(literals)}))
