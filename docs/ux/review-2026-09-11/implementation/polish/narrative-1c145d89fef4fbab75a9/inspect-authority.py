"""Read narrowly scoped production authority and verify the previous live payload."""
import concurrent.futures
import datetime
import hashlib
import json
import subprocess
import urllib.request
from pathlib import Path

work = Path('/tmp/dystrail-narrative-release-prep')
previous = Path('/tmp/dystrail-visual-release')
manifest = json.loads((previous/'release-manifest.json').read_text())
assert manifest['revision'] == 'a9ff83cd924a07fdf4ec', 'Local baseline revision changed'
local = {path.relative_to(previous/'release-site').as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
         for path in (previous/'release-site').rglob('*') if path.is_file()}
assert local == manifest['files'], 'Preserved local payload differs from its release manifest'

def gh(*args):
    return json.loads(subprocess.check_output(['gh', *args], text=True))

pages = gh('api','repos/VannaDii/Dystrail/pages','--jq','{html_url,build_type,status,https_enforced,cname,source}')
runs = gh('run','list','--repo','VannaDii/Dystrail','--workflow','deploy.yml','--limit','3','--json','databaseId,headBranch,headSha,status,conclusion,createdAt,url')
policies = gh('api','repos/VannaDii/Dystrail/environments/github-pages/deployment-branch-policies','--jq','.branch_policies | map({id,name,type})')
authority = {'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'pages':pages,'runs':runs,'branch_policies':policies}
(work/'production-authority.json').write_text(json.dumps(authority,indent=2)+'\n')
assert pages['cname']=='dystrail.com' and pages['html_url']=='https://dystrail.com/' and pages['https_enforced'], 'Production domain authority changed'
assert runs[0]['headSha']=='acaa88b1c1f104bf267048d393df2be403f056c9' and runs[0]['conclusion']=='success', 'Production release baseline changed'
assert policies == [{'id':39031816, 'name':'main', 'type':'branch'}], 'Production branch policy changed'

allowed={'.html':{'text/html'},'.css':{'text/css'},'.js':{'application/javascript','text/javascript'},'.wasm':{'application/wasm'},'.json':{'application/json'},'.png':{'image/png'},'.jpg':{'image/jpeg'},'.webp':{'image/webp'},'.svg':{'image/svg+xml'},'.ico':{'image/x-icon','image/vnd.microsoft.icon'},'.woff2':{'font/woff2','application/font-woff','application/octet-stream'},'.webmanifest':{'application/manifest+json','application/json','application/octet-stream'}}

def check(item):
    name,expected=item
    url='https://dystrail.com/'+name
    request=urllib.request.Request(url,headers={'Cache-Control':'no-cache','Accept-Encoding':'identity'})
    with urllib.request.urlopen(request,timeout=40) as response:
        content=response.read()
        actual=hashlib.sha256(content).hexdigest()
        content_type=response.headers.get('Content-Type','')
        suffix=Path(name).suffix
        type_matches=suffix not in allowed or content_type.split(';')[0] in allowed[suffix]
        return {'path':name,'url':url,'status':response.status,'type':content_type,'bytes':len(content),'sha256':actual,'expected_sha256':expected,'hash_matches':actual==expected,'type_matches':type_matches,'matches':actual==expected and type_matches}

with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
    files=list(pool.map(check,manifest['files'].items()))
failed=[file for file in files if not file['matches']]
report={'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'revision':manifest['revision'],'commit':runs[0]['headSha'],'file_count':len(files),'files':files,'failed':failed,'passed':not failed}
(work/'production-baseline-verification.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'pages':pages,'release':runs[0],'branch_policies':policies,'revision':manifest['revision'],'file_count':len(files),'passed':not failed,'failed':failed},indent=2))
assert not failed, 'Live payload does not match the preserved production baseline'
