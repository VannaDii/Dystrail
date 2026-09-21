"""Join retained originals to explicit historical mappings; do not select artwork."""
import csv,json,re
from pathlib import Path
root=Path(__file__).resolve().parent
rows=list(csv.DictReader((root.parents[1]/'docs/ux/image-production-review-2026-09-21/retained-images.csv').open()))
known={Path(r['source_file']).name:r for r in rows}; mappings={k:[] for k in known}
def source(value):
 if not isinstance(value,str):return None
 m=re.search(r'(?:exec-)?([0-9a-f]{8}-[0-9a-f-]{27})(?:\.png)?',value)
 return 'exec-'+m[1]+'.png' if m and 'exec-'+m[1]+'.png' in known else None
def add(src,alias,evidence):
 if src not in known or not isinstance(alias,str) or source(alias):return
 if alias in ('source','source_image','rejected','replacement','reason','status'):return
 entry={'alias':alias,'evidence':evidence}
 if entry not in mappings[src]:mappings[src].append(entry)
def walk(value,evidence,command=''):
 if isinstance(value,dict):
  src=source(value.get('source_image')) or source(value.get('source'))
  if src:
   for key in ('path','packaged','id'): 
    if isinstance(value.get(key),str):add(src,value[key],evidence)
  for k,v in value.items():
   src=source(v)
   if src:
    if len(k)>1 and k not in ('source','source_image','status','rejected','replacement','reason'):add(src,k,evidence)
    # Expand only simple filename templates with a known variant key.
    if k in ('A','B','C','a','b','c','diner','maps','laptop','memoir'):
     templates=re.findall(r"(?:[a-zA-Z0-9_-]+/)*[a-zA-Z0-9_-]+-\{(?:v|variant|label)(?:\.lower\(\))?\}[a-zA-Z0-9_-]*\.png",command)
     for template in templates:
      alias=re.sub(r'\{[^}]+\}',k.lower() if '.lower()' in template else k,template)
      add(src,alias,evidence)
   walk(v,evidence,command)
 elif isinstance(value,list):
  if len(value)==2 and all(isinstance(x,str) for x in value):
   if source(value[0]):add(source(value[0]),value[1],evidence)
   if source(value[1]):add(source(value[1]),value[0],evidence)
  for v in value:walk(v,evidence,command)
for item in json.loads((root/'recorded-asset-literals.json').read_text()):walk(item['value'],f"session record {item['record']}",item['recorded_command'])
for p in (root/'recorded-files/review/art-satire').glob('*assets*.json'):
 try:walk(json.loads(p.read_text()),str(p.relative_to(root)))
 except json.JSONDecodeError:pass
out=[]
for name,row in known.items():
 aliases=sorted(set(m['alias'] for m in mappings[name]))
 row.update({'durable_original':'/Users/vanna/Source/Dystrail-worktrees/recovery-2026-09-21/generated-originals/'+name,'explicit_historical_aliases':'; '.join(aliases),'mapping_evidence':json.dumps(mappings[name]),'decision':'user_review_existing' if aliases or row['recorded_filename_candidates'] else 'unmapped_existing_hold','new_generation':'none','dimensions':'retain '+row['width_px']+' x '+row['height_px']+'; do not stretch'})
 out.append(row)
with (root/'image-mappings.csv').open('w') as f:
 w=csv.DictWriter(f,fieldnames=out[0]);w.writeheader();w.writerows(out)
(root/'image-mappings.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'total':len(out),'explicit_mapping':sum(bool(r['explicit_historical_aliases']) for r in out),'any_mapping':sum(bool(r['explicit_historical_aliases'] or r['recorded_filename_candidates']) for r in out),'new_generation_requests':0}))
