"""Stage the CI-built artifact for verification; publication stays held until all source checks pass."""
import hashlib,json,subprocess
from pathlib import Path
work=Path(__file__).resolve().parent
out=work.parent
source='d7b22770302a2149392a5d34aa981b1f6141087b'
run_id=34798552993
checkout=Path('/tmp/dystrail-source-handoff-publication')
repo='VannaDii/Dystrail'
artifact=out/'ci-chevron-build'
def command(*args,cwd=None):
 return subprocess.check_output(args,cwd=cwd,text=True)
run=json.loads(command('gh','run','view',str(run_id),'--repo',repo,'--json','headSha,status,conclusion,url,jobs'))
assert run['headSha']==source and (run['status']!='completed' or run['conclusion']=='success')
assert any(job['name']=='build' and job['conclusion']=='success' for job in run['jobs']), 'The exact source artifact must be built successfully'
(work/'source-ci-verification.json').write_text(json.dumps(run,indent=2)+'\n')
assert not command('git','status','--porcelain',cwd=checkout)
assert command('git','rev-parse','HEAD',cwd=checkout).strip()=='5722f8b0b36b27c815902359a54a08e77e7c6589'
command('gh','run','download',str(run_id),'--repo',repo,'--name','dystrail-web-dist','--dir',str(artifact))
data=(artifact/'offline-manifest.json').read_bytes();manifest=json.loads(data)
subprocess.run(['python3',str(work/'stage-artifact.py'),'--artifact',str(artifact),'--revision',manifest['revision'],'--manifest-sha256',hashlib.sha256(data).hexdigest(),'--source-commit',source,'--source-ci-run',str(run_id)],check=True)
subprocess.run(['python3','scripts/verify-release-payload.py'],cwd=checkout,check=True)
verification=out/'chevron-saved-install/verification.json'
subprocess.run(['node',str(work/'verify-update.mjs'),str(artifact),manifest['revision'],str(verification.parent)],check=True)
subprocess.run(['python3','/tmp/dystrail-share-hud-review/update-preview.py',str(artifact),str(out/'preview-chevron-ci.json')],check=True)
record={'revision':manifest['revision'],'source_commit':source,'source_ci_commit':source,'source_ci':run_id,'release_commit':None,'dispatch_on_hold':True,'parent_verified_signal_received':False,'saved_install_verification':str(verification)}
(work/'staged-release-record.json').write_text(json.dumps(record,indent=2)+'\n')
print('Prepared and verified exact CI artifact',manifest['revision'],flush=True)
