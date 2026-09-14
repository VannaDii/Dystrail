"""Prepare the exact current CI artifact, verify it, and finish the authorized release."""
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path

work = Path(__file__).resolve().parent
task = work.parent
checkout = Path('/tmp/dystrail-source-handoff-publication')
repo = 'VannaDii/Dystrail'
run_id = int(sys.argv[1])
source = sys.argv[2]
artifact = task/'ci-geometry-build'

def command(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True)

def gh(*args):
    return json.loads(command('gh', *args))

record = json.loads((work/'staged-release-record.json').read_text())
record.update(source_commit=source, source_ci_commit=source, source_ci=run_id,
              release_commit=None, dispatch_on_hold=True,
              hold_reason='Waiting for the current CI artifact and all checks.')
(work/'staged-release-record.json').write_text(json.dumps(record,indent=2)+'\n')
print('Waiting for the exact current CI artifact.',flush=True)
for _ in range(60):
    run = gh('run','view',str(run_id),'--repo',repo,'--json','headSha,status,conclusion')
    assert run['headSha']==source
    assert run['status']!='completed' or run['conclusion']=='success', run
    artifacts = gh('api',f'repos/{repo}/actions/runs/{run_id}/artifacts')['artifacts']
    found = next((a for a in artifacts if a['name']=='dystrail-web-dist' and not a['expired']),None)
    if found:
        break
    time.sleep(30)
else:
    raise RuntimeError('No final artifact within 30 minutes; publication remains held.')

command('gh','run','download',str(run_id),'--repo',repo,'--name','dystrail-web-dist','--dir',str(artifact))
data = (artifact/'offline-manifest.json').read_bytes()
manifest = json.loads(data)
assert not command('git','status','--porcelain',cwd=checkout)
old_release = json.loads((checkout/'release-manifest.json').read_text())
actual = {p.relative_to(checkout/'release-site').as_posix():hashlib.sha256(p.read_bytes()).hexdigest()
          for p in (checkout/'release-site').rglob('*') if p.is_file()}
assert actual==old_release['files']
command('git','restore','--source=99ae520b19f1ef8eb259d2211229fe954e4d3803','--','release-site',cwd=checkout)
subprocess.run(['python3',str(work/'stage-artifact.py'),'--artifact',str(artifact),
                '--revision',manifest['revision'],'--manifest-sha256',hashlib.sha256(data).hexdigest(),
                '--source-commit',source,'--source-ci-run',str(run_id)],check=True)
subprocess.run(['python3','scripts/verify-release-payload.py'],cwd=checkout,check=True)

verification = task/'readiness-update-check/verification.json'
same_bytes = data==(task/'ci-readiness-build/offline-manifest.json').read_bytes()
if same_bytes:
    assert json.loads(verification.read_text())['passed']
    print('Runtime matches the already verified artifact byte for byte; saved-install evidence remains applicable.',flush=True)
else:
    verification = task/'geometry-update-check/verification.json'
    subprocess.run(['node',str(task/'verify-update.mjs'),str(artifact),manifest['revision'],str(verification.parent)],check=True)

subprocess.run(['python3','/tmp/dystrail-share-hud-review/update-preview.py',str(artifact),str(task/'latest-ci-preview.json')],check=True)
command('git','add','--','release-site','release-manifest.json','scripts/verify-release-payload.py',cwd=checkout)
command('git','diff','--cached','--check',cwd=checkout)
command('git','commit','-m','chore: record final release build provenance',cwd=checkout)
command('git','push','origin','release/source-handoff-publication-2026-09-13',cwd=checkout)
release_commit = command('git','rev-parse','HEAD',cwd=checkout).strip()
record.update(revision=manifest['revision'],release_commit=release_commit,
              artifact=str(found['id']),dispatch_on_hold=False,hold_reason=None,
              saved_install_verification=str(verification),runtime_matches_prior_verified_artifact=same_bytes)
(work/'staged-release-record.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({'prepared_revision':manifest['revision'],'release_commit':release_commit,'source_ci':run_id}),flush=True)
subprocess.run(['python3',str(work/'finish-release.py')],check=True)
