"""Wait for the exact source CI, then publish and verify the reviewed package."""
import json, subprocess, time
from pathlib import Path
work=Path(__file__).resolve().parent
authorization=json.loads((work/'staged-release-record.json').read_text())
last=None
for attempt in range(60):
 raw=subprocess.check_output(['gh','run','view',str(authorization['source_ci']),'--repo','VannaDii/Dystrail','--json','headSha,status,conclusion,url,jobs'],text=True)
 run=json.loads(raw)
 assert run['headSha']==authorization['source_ci_commit']
 state=[(j['name'],j['status'],j['conclusion']) for j in run['jobs']]
 if state!=last:print(json.dumps({'source_ci':run['url'],'jobs':state}),flush=True);last=state
 if run['status']=='completed':
  (work/'source-ci-verification.json').write_text(json.dumps(run,indent=2)+'\n')
  assert run['conclusion']=='success', 'Source CI did not pass; publication remains on hold'
  subprocess.run(['python3',str(work/'publish-verified-release.py')],check=True)
  subprocess.run(['python3',str(work/'verify-live-release.py')],check=True)
  break
 time.sleep(30)
else:raise RuntimeError('Source CI has not completed within 30 minutes; no publication attempted')
