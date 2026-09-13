"""Publish the reviewed share-card and status-layout artifact; always remove its exact temporary policy."""
import datetime
import json
import subprocess
import time
from pathlib import Path

out = Path('/tmp/dystrail-share-hud-review/release-prep')
workspace = Path('/tmp/dystrail-share-card-hud-release')
repo = 'VannaDii/Dystrail'
branch = 'release/share-card-hud-2026-09-13'
commit = '27f71c1b7f448487da07dced09c61a9b8804903a'
expected_policy = [{'id':39031816, 'name':'main', 'type':'branch'}]
policy_endpoint = f'repos/{repo}/environments/github-pages/deployment-branch-policies'
record = {'revision':'33dd2b05a6658d330912','branch':branch,'commit':commit,'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat()}

def command(*args):
    return subprocess.check_output(['gh', *args], text=True, cwd=workspace)

def gh(*args):
    return json.loads(command(*args))

def policies():
    return gh('api',policy_endpoint,'--jq','.branch_policies | map({id,name,type})')

def save():
    (out/'deployment-run.json').write_text(json.dumps(record,indent=2)+'\n')

authorization = json.loads((out/'staged-release-record.json').read_text())
assert authorization['parent_verified_signal_received'] and not authorization['dispatch_on_hold']
assert authorization['revision'] == record['revision']
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=workspace,text=True).strip() == commit
assert not subprocess.check_output(['git','status','--porcelain'],cwd=workspace,text=True)
subprocess.run(['python3','scripts/verify-release-payload.py'],cwd=workspace,check=True)
assert gh('api',f'repos/{repo}/commits/{branch}','--jq','{sha}')['sha'] == commit
assert policies() == expected_policy
latest = gh('run','list','--repo',repo,'--workflow','deploy.yml','--limit','1','--json','headSha,status,conclusion')
assert latest == [{'conclusion':'success','headSha':'ec2dc29eef5125eb04d737e1ae569754b0a98d7f','status':'completed'}], 'Production baseline changed before dispatch'
temporary = None
try:
    temporary = gh('api','--method','POST',policy_endpoint,'-f',f'name={branch}','-f','type=branch')
    assert temporary['name'] == branch and temporary['type'] == 'branch'
    record['temporary_policy'] = temporary
    (out/'temporary-policy.json').write_text(json.dumps(temporary,indent=2)+'\n')
    save()
    print('Exact release branch permitted; dispatching reviewed commit',commit,flush=True)
    command('workflow','run','deploy.yml','--repo',repo,'--ref',branch)
    record['dispatched_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    save()
    deadline = time.monotonic()+20*60
    run = None
    last_status = None
    while time.monotonic()<deadline:
        if run is None:
            runs = gh('run','list','--repo',repo,'--workflow','deploy.yml','--branch',branch,'--limit','5','--json','databaseId,headBranch,headSha,status,conclusion,createdAt,url')
            run = next((item for item in runs if item['headSha']==commit),None)
            if run is None:
                time.sleep(5)
                continue
        current = gh('run','view',str(run['databaseId']),'--repo',repo,'--json','databaseId,headBranch,headSha,status,conclusion,createdAt,url,jobs')
        assert current['headSha']==commit and current['headBranch']==branch
        record['run'] = current
        save()
        status = (current['status'],current['conclusion'],[(job['name'],job['status'],job['conclusion']) for job in current['jobs']])
        if status != last_status:
            print(json.dumps({'url':current['url'],'status':status}),flush=True)
            last_status = status
        if current['status']=='completed':
            assert current['conclusion']=='success', current
            record['deployed_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
            save()
            break
        time.sleep(10)
    else:
        raise RuntimeError('Timed out waiting for the exact release workflow')
except BaseException as error:
    record['failure'] = str(error)
    save()
    raise
finally:
    if temporary is not None:
        command('api','--method','DELETE',policy_endpoint+'/'+str(temporary['id']))
        record['temporary_policy_removed'] = True
    after = policies()
    (out/'branch-policies-after.json').write_text(json.dumps(after,indent=2)+'\n')
    record['branch_policies_after'] = after
    save()
    assert after == expected_policy, 'Original main-only deployment policy was not restored'
    print('Temporary exact-branch policy removed; main-only policy restored.',flush=True)
