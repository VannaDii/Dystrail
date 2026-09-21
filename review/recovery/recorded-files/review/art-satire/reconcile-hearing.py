"""Import a verified snapshot of the completed hearing task without editing its checkout."""
import hashlib,json,pathlib,subprocess

source=pathlib.Path('/Users/vanna/Source/Dystrail')
target=pathlib.Path('/private/tmp/dystrail-art-satire')
base='f1ed4fe'
out=target/'review/art-satire/hearing-snapshot'
out.mkdir(parents=True,exist_ok=True)
def git(*args): return subprocess.check_output(['git','-C',str(source),*args])
paths=git('diff','--name-only',base).decode().splitlines()
paths=[p for p in paths if p!='progress.md' and not p.startswith('docs/')]
extra=['dystrail-web/src/components/ui/character_portrait.rs','dystrail-web/static/hearing.css','dystrail-web/tests-e2e/hearing.spec.ts']
for directory in ['dystrail-game/src/boss','dystrail-web/src/pages/boss']:
 extra += [str(p.relative_to(source)) for p in (source/directory).rglob('*') if p.is_file()]
extra += [str(p.relative_to(source)) for p in (source/'dystrail-web/static/img/journey').glob('*expressions-v1.png')]
extra += ['dystrail-web/static/img/journey/hearing-officials-v1.png']
paths=sorted(set(paths+extra))
snapshot={p:(source/p).read_bytes() for p in paths}
assert all((source/p).read_bytes()==data for p,data in snapshot.items()),'Source changed during snapshot'
report={'source_base':base,'files':[],'conflicts':[],'retained_feature_override':['dystrail-web/src/components/ui/journey_scene/parked.rs']}
def merge_json(old,ours,theirs,path=''):
 if ours==old:return theirs
 if theirs==old or ours==theirs:return ours
 if all(isinstance(v,dict) for v in (old,ours,theirs)):
  result={}
  missing=object()
  for k in sorted(set(old)|set(ours)|set(theirs)):
   a,b,c=(v.get(k,missing) for v in (old,ours,theirs))
   if b is missing and c is missing:continue
   if b is missing and a is missing:result[k]=c
   elif c is missing and a is missing:result[k]=b
   elif b is missing and c==a:continue
   elif c is missing and b==a:continue
   elif a is missing and b==c:result[k]=b
   else:result[k]=merge_json({} if a is missing else a,b,c,f'{path}.{k}')
  return result
 raise ValueError(f'Conflicting locale field {path}: {ours!r} / {theirs!r}')
for name,data in snapshot.items():
 saved=out/name;saved.parent.mkdir(parents=True,exist_ok=True);saved.write_bytes(data)
 report['files'].append({'path':name,'sha256':hashlib.sha256(data).hexdigest()})
 if name in report['retained_feature_override']:continue
 dest=target/name;dest.parent.mkdir(parents=True,exist_ok=True)
 try:old=git('show',f'{base}:{name}')
 except subprocess.CalledProcessError:old=b''
 ours=dest.read_bytes() if dest.exists() else b''
 if ours==old or not ours:dest.write_bytes(data);continue
 if data==old or ours==data:continue
 if name.startswith('dystrail-web/i18n/'):
  merged=merge_json(json.loads(old),json.loads(ours),json.loads(data))
  dest.write_text(json.dumps(merged,ensure_ascii=False,indent=2)+'\n');continue
 basefile=out/'merge-base';oursfile=out/'merge-ours';basefile.write_bytes(old);oursfile.write_bytes(ours)
 result=subprocess.run(['git','merge-file','-p','--diff3',str(oursfile),str(basefile),str(saved)],capture_output=True)
 dest.write_bytes(result.stdout)
 if result.returncode:report['conflicts'].append(name)
(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'files':len(paths),'conflicts':report['conflicts']},indent=2))
