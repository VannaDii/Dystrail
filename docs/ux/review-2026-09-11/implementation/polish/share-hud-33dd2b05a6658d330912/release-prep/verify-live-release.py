"""After publication, verify every live byte and response type against the frozen release."""
import concurrent.futures
import hashlib
import json
import time
import urllib.request
from pathlib import Path

work=Path('/tmp/dystrail-share-hud-review/release-prep')
manifest=json.loads(Path('/tmp/dystrail-share-card-hud-release/release-manifest.json').read_text())
base='https://dystrail.com/'

def fetch(url):
    request=urllib.request.Request(url,headers={'Cache-Control':'no-cache','Accept-Encoding':'identity'})
    with urllib.request.urlopen(request,timeout=45) as response:
        data=response.read()
        return data,{'url':url,'final_url':response.url,'status':response.status,'type':response.headers.get('Content-Type',''),'cache_control':response.headers.get('Cache-Control',''),'age':response.headers.get('Age'),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}

deadline=time.monotonic()+20*60
while time.monotonic()<deadline:
    _,worker=fetch(base+'play/sw.js')
    if worker['sha256']==manifest['files']['play/sw.js']:
        _,index=fetch(base+'play/index.html')
        _,offline=fetch(base+'play/offline-manifest.json')
        if index['sha256']==manifest['files']['play/index.html'] and offline['sha256']==manifest['files']['play/offline-manifest.json']:
            break
    time.sleep(5)
else:raise RuntimeError('Frozen production worker, index and manifest were not all published')
print('Frozen game worker, index and offline manifest are available',manifest['revision'],flush=True)
allowed={'':{'application/octet-stream'},'.txt':{'text/plain'},'.md':{'text/markdown','text/plain'},'.toml':{'application/toml','text/plain'},'.xml':{'application/xml','text/xml'},'.html':{'text/html'},'.css':{'text/css'},'.js':{'application/javascript','text/javascript'},'.wasm':{'application/wasm'},'.json':{'application/json'},'.png':{'image/png'},'.jpg':{'image/jpeg'},'.webp':{'image/webp'},'.svg':{'image/svg+xml'},'.ico':{'image/x-icon','image/vnd.microsoft.icon'},'.woff2':{'font/woff2','application/font-woff','application/octet-stream'},'.webmanifest':{'application/manifest+json','application/json','application/octet-stream'}}
def check(item):
    name,expected=item
    _,info=fetch(base+name)
    info['path']=name
    info['hash_matches']=info['sha256']==expected
    suffix=Path(name).suffix
    assert suffix in allowed, f'Missing MIME policy for {name}'
    info['type_matches']=info['type'].split(';')[0] in allowed[suffix]
    return info
with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
    files=list(pool.map(check,manifest['files'].items()))
failed=[info for info in files if not info['hash_matches'] or not info['type_matches']]
canonical=[]
canonical_expected=[
    ('https://dystrail.com/','https://dystrail.com/','index.html'),
    ('https://dystrail.com/play/','https://dystrail.com/play/','play/index.html'),
    ('https://dystrail.com/docs/','https://dystrail.com/docs/','docs/index.html'),
    ('http://dystrail.com/play/','https://dystrail.com/play/','play/index.html'),
    ('https://www.dystrail.com/play/','https://dystrail.com/play/','play/index.html'),
    ('https://dystrail.com/styles.css?v=a0a01e7e7a882120','https://dystrail.com/styles.css?v=a0a01e7e7a882120','styles.css'),
]
for url,final_url,path in canonical_expected:
    _,info=fetch(url)
    info['matches']=info['status']==200 and info['final_url']==final_url and info['sha256']==manifest['files'][path] and info['type'].split(';')[0] in allowed[Path(path).suffix]
    canonical.append(info)
failed.extend(info for info in canonical if not info['matches'])
report={'revision':manifest['revision'],'files':files,'canonical':canonical,'failed':failed,'preserved_nonplay_files':len(manifest['preserved_files']),'passed':not failed}
(work/'production-http-verification.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'revision':manifest['revision'],'files':len(files),'preserved_nonplay_files':report['preserved_nonplay_files'],'passed':report['passed'],'failed':failed,'canonical':canonical},indent=2),flush=True)
assert not failed, 'One or more live payload files did not match the frozen release'
