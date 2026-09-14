"""Commit, push and publish the reviewed payload only after every source gate succeeds."""
import json,subprocess
from pathlib import Path
work=Path(__file__).resolve().parent
checkout=Path('/tmp/dystrail-source-handoff-publication')
record=json.loads((work/'staged-release-record.json').read_text())
def command(*args,cwd=None):
 return subprocess.check_output(args,cwd=cwd,text=True)
run=json.loads(command('gh','run','view',str(record['source_ci']),'--repo','VannaDii/Dystrail','--json','headSha,status,conclusion,url,jobs'))
assert run['headSha']==record['source_ci_commit'] and run['status']=='completed' and run['conclusion']=='success'
assert all(job['conclusion']=='success' for job in run['jobs'])
(work/'source-ci-verification.json').write_text(json.dumps(run,indent=2)+'\n')
verification=json.loads(Path(record['saved_install_verification']).read_text())
assert verification['passed'] and verification['expected']==record['revision']
subprocess.run(['python3','scripts/verify-release-payload.py'],cwd=checkout,check=True)
if not record['release_commit']:
 assert command('git','rev-parse','HEAD',cwd=checkout).strip()=='5722f8b0b36b27c815902359a54a08e77e7c6589'
 command('git','add','--','release-site','release-manifest.json','scripts/verify-release-payload.py',cwd=checkout)
 command('git','diff','--cached','--check',cwd=checkout)
 print(command('git','commit','-m','fix: publish floating language menu and reliable game entry',cwd=checkout),flush=True)
 record['release_commit']=command('git','rev-parse','HEAD',cwd=checkout).strip()
 (work/'staged-release-record.json').write_text(json.dumps(record,indent=2)+'\n')
assert command('git','rev-parse','HEAD',cwd=checkout).strip()==record['release_commit']
assert not command('git','status','--porcelain',cwd=checkout)
command('git','push','origin','release/source-handoff-publication-2026-09-13',cwd=checkout)
record.update(dispatch_on_hold=False,parent_verified_signal_received=True)
(work/'staged-release-record.json').write_text(json.dumps(record,indent=2)+'\n')
subprocess.run(['python3',str(work/'publish-verified-release.py')],check=True)
subprocess.run(['python3',str(work/'verify-live-release.py')],check=True)
