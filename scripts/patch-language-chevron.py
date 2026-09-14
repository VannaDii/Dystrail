"""Apply the language-chevron CSS hotfix to an otherwise unchanged release."""
import base64, hashlib, json, re
from pathlib import Path
root=Path(__file__).resolve().parents[1]
payload=root/'release-site'; play=payload/'play'
release=json.loads((root/'release-manifest.json').read_text())
old_revision=release['revision']
index=(play/'index.html').read_text()
old_name=re.search(r'/play/(journey-[a-f0-9]+\.css)',index)[1]
old_css=(play/old_name).read_text()
old='.top-game-menu-panel .language-picker>button{width:100%;text-align:start}'
new='.top-game-menu-panel .language-picker>button{width:100%;text-align:start;display:flex;align-items:center;justify-content:space-between;gap:.75rem}\n.top-game-menu-panel .language-picker>button>span{flex-shrink:0}'
assert old_css.count(old)==1
css=old_css.replace(old,new).encode()
name='journey-'+hashlib.sha256(css).hexdigest()[:16]+'.css'
(play/name).write_bytes(css)
old_sri='sha384-'+base64.b64encode(hashlib.sha384((play/old_name).read_bytes()).digest()).decode()
sri='sha384-'+base64.b64encode(hashlib.sha384(css).digest()).decode()
assert old_sri in index
index=index.replace(old_name,name).replace(old_sri,sri)
(play/'index.html').write_text(index); (payload/'404.html').write_text(index)
manifest=json.loads((play/'offline-manifest.json').read_text())
for asset in manifest['assets']:
    if asset['path']==old_name: asset['path']=name
    data=(play/asset['path']).read_bytes()
    asset['bytes']=len(data)
    asset['integrity']='sha256-'+base64.b64encode(hashlib.sha256(data).digest()).decode()
manifest['assets'].sort(key=lambda a:a['path'])
manifest['revision']=hashlib.sha256(json.dumps(manifest['assets'],sort_keys=True).encode()).hexdigest()[:20]
manifest['bytes']=sum(a['bytes'] for a in manifest['assets'])
(play/'offline-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
worker=(play/'sw.js').read_text()
worker=re.sub(r'^const BUILD = .*;$','const BUILD = '+json.dumps(manifest)+';',worker,count=1,flags=re.M)
(play/'sw.js').write_text(worker)
release['previous_revision']=old_revision; release['revision']=manifest['revision']
release['files']={p.relative_to(payload).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(payload.rglob('*')) if p.is_file()}
release['game_assets']=len(manifest['assets']); release['game_bytes']=manifest['bytes']
release['artifact_manifest_sha256']=hashlib.sha256((play/'offline-manifest.json').read_bytes()).hexdigest()
release['build_origin']='CSS-only language chevron hotfix over production release 46a43c6efd6ba0b6d91f1d816c43846db942363e; base source CI provenance retained'
release['hotfix']={'description':'Align language chevron at the trailing edge with existing button padding','baseline_release':'46a43c6efd6ba0b6d91f1d816c43846db942363e','stylesheet':name}
(root/'release-manifest.json').write_text(json.dumps(release,indent=2)+'\n')
v=root/'scripts/verify-release-payload.py'
v.write_text(v.read_text().replace(old_revision,manifest['revision']))
print(json.dumps({'revision':manifest['revision'],'stylesheet':name}))
