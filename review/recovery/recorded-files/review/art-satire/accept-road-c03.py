"""Record the reviewed C03 integration; do not treat this as feature completion."""
import csv,hashlib,json
from collections import Counter
from pathlib import Path
root=Path(__file__).resolve().parents[2]; review=root/'review/art-satire'
catalog_path=review/'asset-catalog.json'; catalog=json.loads(catalog_path.read_text())
corrections=json.loads((review/'road-c03-corrections.json').read_text())
for variant,source in corrections['acceptedSources'].items():
 id=f'road-c03-{variant.lower()}-20260914';relative=f'dystrail-web/static/img/scenes-v2/{id}.png';raw=(root/relative).read_bytes()
 assert raw==Path(source).read_bytes()
 entry={'id':id,'path':relative,'source_image':source,'prompt':'review/art-satire/road-c03-generation-draft.json','correction_prompts':'review/art-satire/road-c03-corrections.json','sha256':hashlib.sha256(raw).hexdigest(),'method':'built-in image generation','background':'opaque','status':'integrated_render_reviewed','source_revision':'source-refresh-20260914','row_boundaries':[0,512,1024],'column_boundaries':[0,768,1536],'source_inset':4,'religious_symbol_review':'All four source frames and representative desktop/mobile compositions reviewed; no religious imagery found.'}
 catalog['assets']=[a for a in catalog['assets'] if a['id']!=id]+[entry]
for u in catalog['units']:
 if u['id'].startswith('ENC-C03-'):
  u['production_status']='integrated_render_reviewed'
  u['asset']={'atlas':f"road-c03-{u['variant'].lower()}-20260914",'offer_cell':0,'choice_cells':[1,2,3]}
  u['art_constraint']='No religious identifiers. Ordinary feminine/gender-neutral locals remain separate from the active crew portrait. A window view uses generic trees; B offer keeps the chair outside the dry corner until donated food is committed; B copy describes residual leaking so weather is not falsely fixed; C phone/quiet-room staging and pronouns follow the later cast brief.'
catalog_path.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
audit_path=review/'religious-symbol-audit/source-review.json'; audit=json.loads(audit_path.read_text())
for variant,source in corrections['acceptedSources'].items():
 name=f'scenes-v2/road-c03-{variant.lower()}-20260914.png'
 audit=[a for a in audit if a['asset']!=name]+[{'asset':name,'sha256':hashlib.sha256(Path(source).read_bytes()).hexdigest(),'source_review':'All four source frames reviewed; no religious symbol found','render_review':'Representative desktop/mobile and Arabic offer/outcome compositions reviewed; evidence road-c03-final-results'}]
audit_path.write_text(json.dumps(audit,indent=2)+'\n')

# Refresh the original coverage sheet from the reconciled pack, retaining all IDs
# and its source-release membership rather than confusing that with current runtime.
pack={u['id']:u for u in json.loads((review/'source-refresh-20260914/complete-pack.json').read_text())['units']}
current={u['id']:u for u in catalog['units']}
p=root/'docs/ux/enhancement-review-2026-09-13/features/art-and-satire/satire-art-coverage.csv'
with p.open(newline='') as f:reader=csv.DictReader(f);fields=reader.fieldnames;rows=list(reader)
assert len(rows)==644 and {r['unit_id'] for r in rows}==set(pack)==set(current)
for row in rows:
 u=pack[row['unit_id']]; scene=u.get('scene') or {}
 row['source_scene_id']=u.get('scene_id',scene.get('scene_id',''))
 row['scene_direction']=scene.get('setting',scene.get('action','')) if isinstance(scene,dict) else str(scene)
 row['overlay_count']=len(scene.get('overlays',[])) if isinstance(scene,dict) else 0
 row['asset_mapping_status']=current[row['unit_id']]['production_status']
with p.open('w',newline='') as f:writer=csv.DictWriter(f,fieldnames=fields);writer.writeheader();writer.writerows(rows)
counts=Counter(u['production_status'] for u in catalog['units'])
record={'build':'8ca74dab48559e9a3f50','units':['ENC-C03-A','ENC-C03-B','ENC-C03-C'],'frames':12,'runtime':'classic_crossing_block_party','status':'integrated_render_reviewed','locales':['en','es','it','ar'],'other_locales':'Explicit English narrative fallback; unrelated locale fields preserved.','sources':'road-c03-source-refresh.json; live document C03 text matched the reconciled pack.','verification':{'native_tests':100,'native_clippy':'passed','wasm_clippy':'passed','first_browser_build':'fb39bff1bbb711dcb2f6','first_browser_cases':4,'final_variant_cases':2,'final_accessibility_cases':2,'variant_actions_per_platform':36,'offline_assets':158,'hearing_files_preserved':7,'hearing_locale_subtrees_preserved':20,'main_source_files_preserved':60},'render_review':['All source frames inspected at native size for staging, cast, pixel style and religious exclusion.','Desktop/phone English and Arabic offers/outcomes inspected; A recording-frame label was moved and narrowed after the first review.','Instant scroll in capture helper removes smooth-scroll sticky-header artifacts without changing the game.','Small prop lettering has a visible localized HTML transcript.','C03 enlarged Arabic captions and choice controls, reduced motion and keyboard activation passed.'],'remaining_findings':['At 200 percent text enlargement, the shared HUD labels and historical resource receipt values still crowd or overlap. The new C03 caption/choice checks do not establish full-page accessibility; retain this for the broader layout acceptance work.','Full feature remains in production; remaining variants, derived cast assets, final performance/accessibility/route coverage and combined hearing release checks remain.'],'catalog_counts':dict(counts)}
(review/'road-c03-review.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'catalog_records':len(catalog['units']),'counts':dict(counts),'source_audit_images':len(audit),'coverage_rows':len(rows)}))
