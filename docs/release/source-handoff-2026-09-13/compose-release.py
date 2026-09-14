"""Combine the CSS fix with the unchanged, already verified production runtime."""
import base64, hashlib, json, re, shutil
from pathlib import Path
root=Path('/Users/vanna/Source/Dystrail')
work=Path('/tmp/dystrail-source-handoff-20260913')
old=Path('/tmp/dystrail-native-share-focus/web')
built=work/'rebuilt-web'
out=work/'release-web'
assert not out.exists()
shutil.copytree(old,out)
old_css=next(old.glob('continuity-*.css'));new_css=next(built.glob('continuity-*.css'))
assert new_css.read_bytes()==(root/'dystrail-web/static/continuity.css').read_bytes()
html=(out/'index.html').read_text()
old_sri='sha384-'+base64.b64encode(hashlib.sha384(old_css.read_bytes()).digest()).decode()
new_sri='sha384-'+base64.b64encode(hashlib.sha384(new_css.read_bytes()).digest()).decode()
assert html.count(old_css.name)==1 and html.count(old_sri)==1
(out/'index.html').write_text(html.replace(old_css.name,new_css.name).replace(old_sri,new_sri))
(out/old_css.name).unlink()
shutil.copy2(new_css,out/new_css.name)
shutil.copy2(new_css,out/'static/continuity.css')
assets=[]
for p in sorted(out.rglob('*')):
 if p.is_file() and p.name not in {'sw.js','offline-manifest.json'}:
  b=p.read_bytes();assets.append({'path':p.relative_to(out).as_posix(),'bytes':len(b),'integrity':'sha256-'+base64.b64encode(hashlib.sha256(b).digest()).decode()})
revision=hashlib.sha256(json.dumps(assets,sort_keys=True).encode()).hexdigest()[:20]
m={'revision':revision,'bytes':sum(a['bytes'] for a in assets),'assets':assets}
(out/'offline-manifest.json').write_text(json.dumps(m,indent=2)+'\n')
(out/'sw.js').write_text('const BUILD = '+json.dumps(m)+';\n'+(root/'scripts/offline-worker.js').read_text())
for p in out.glob('dystrail-web-*'):assert p.read_bytes()==(old/p.name).read_bytes()
report={'revision':revision,'source_commit':'3525f031fca6d2d5d22464a20f6bb1d39c57a32a','runtime_baseline':'d3ed78dac458b64a80ee','runtime_bytes_preserved':True,'reason':'CSS-only change. wasm-bindgen reordered generated closures during a fresh build while retaining the same Trunk filenames; retain the exact verified runtime bytes and immutable asset URLs. Runtime source inputs remain identical.','css':new_css.name,'assets':len(assets),'manifest_sha256':hashlib.sha256((out/'offline-manifest.json').read_bytes()).hexdigest()}
(work/'release-composition.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
