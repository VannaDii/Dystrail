"""Record C06 acceptance without treating the unfinished feature as complete."""
import csv, hashlib, json
from collections import Counter
from pathlib import Path

root=Path(__file__).resolve().parents[2]
review=root/'review/art-satire'
assert '6 passed' in (review/'road-c06-tests.log').read_text()
assert '4 passed' in (review/'road-c06-hearing-tests.log').read_text()
assert '100 passed; 0 failed' in (review/'road-c06-native.log').read_text()
manifest=json.loads(Path('/private/tmp/dystrail-art-satire-build/offline-manifest.json').read_text())
assert manifest['revision']=='b71945cdaca36d82c3c3'
catalog_path=review/'asset-catalog.json'
catalog=json.loads(catalog_path.read_text())
assets=json.loads((review/'road-c06-source-assets.json').read_text())
audit_path=review/'religious-symbol-audit/source-review.json'
audit=json.loads(audit_path.read_text())
for variant,source in assets.items():
    raw=(root/source['packaged']).read_bytes()
    assert raw==Path(source['source']).read_bytes()
    id=f'road-c06-{variant.lower()}-20260914'
    digest=hashlib.sha256(raw).hexdigest()
    assert digest==source['sha256']
    entry={'id':id,'path':source['packaged'],'source_image':source['source'],'prompt':'review/art-satire/road-c06-generation-prompts.json','correction_prompts':'review/art-satire/road-c06-corrections.json','sha256':digest,'method':'built-in image generation','background':'opaque','status':'integrated_render_reviewed','source_revision':'source-refresh-20260914','row_boundaries':[0,512,1024],'column_boundaries':[0,768,1536],'source_inset':4,'runtime_cells':[0,1,2],'unused_reference_cell':3,'religious_symbol_review':'All source frames and representative desktop/mobile compositions inspected; no religious imagery found.'}
    catalog['assets']=[a for a in catalog['assets'] if a['id']!=id]+[entry]
    name=f'scenes-v2/{id}.png'
    audit=[a for a in audit if a['asset']!=name]+[{'asset':name,'sha256':digest,'source_review':'All four source frames inspected; no religious symbol found. Functional ladder/ceiling joins are not emblems.','render_review':'Desktop/mobile and Arabic offer/outcome compositions inspected; evidence road-c06-results and road-c06-client-check.'}]
for u in catalog['units']:
    if u['id'].startswith('ENC-C06-'):
        u['production_status']='integrated_render_reviewed'
        u['asset']={'atlas':f"road-c06-{u['variant'].lower()}-20260914",'offer_cell':0,'choice_cells':[1,2],'unused_reference_cell':3}
        u['art_constraint']='Feminine/neutral fictional locals; no religious identifiers, flags or emblems. Active crew remains separate. A retains one missing rung until checking and a separate safe stepladder for rehearsal. B leaves the player chair empty and restores the wage column only after checking. C catches the existing leak, with the emptied bucket and mop salute in their actual outcomes. Native localized titles appear only on offered A/C signs. Interiors remain sheltered; job-fair precipitation stays behind foreground.'
assert len(catalog['units'])==644
catalog_path.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
audit_path.write_text(json.dumps(audit,indent=2)+'\n')
coverage=root/'docs/ux/enhancement-review-2026-09-13/features/art-and-satire/satire-art-coverage.csv'
with coverage.open(newline='') as f:
    reader=csv.DictReader(f);fields=reader.fieldnames;rows=list(reader)
current={u['id']:u for u in catalog['units']}
assert len(rows)==644 and {r['unit_id'] for r in rows}==set(current)
for row in rows:row['asset_mapping_status']=current[row['unit_id']]['production_status']
with coverage.open('w',newline='') as f:
    writer=csv.DictWriter(f,fieldnames=fields);writer.writeheader();writer.writerows(rows)
counts=Counter(u['production_status'] for u in catalog['units'])
record={'build':manifest['revision'],'units':['ENC-C06-A','ENC-C06-B','ENC-C06-C'],'runtime':'classic_media_training','status':'integrated_render_reviewed','gameplay_frames':9,'unused_offer_reference_frames':3,'locales':['en','es','it','ar'],'other_locales':'Explicit English narrative fallback; unrelated locale values preserved.','source_refresh':'road-c06-source-refresh.json','verification':{'native_tests':100,'native_clippy':'passed','wasm_clippy':'passed','desktop_mobile_cases':6,'variant_language_choice_actions_per_platform':24,'hearing_regression_cases':4,'offline_assets':167,'offline_bytes':manifest['bytes'],'hearing_engine_files_preserved':7,'hearing_locale_subtrees_preserved':20,'main_source_snapshot_files_preserved':60,'supplied_game_client':'road-c06-client-check','format_and_diff_checks':'passed'},'render_review':['Source staging corrected for single missing rung and water reaching the bucket. Three runtime cells reflect offer, checking or rehearsal; spare cell never activated.','No religious imagery found in source or representative rendered scenes. Fictional locals remain separate from surviving crew portraits.','Editable A/C titles fit their signs, with visible localized captions. B uses no sign lettering. Desktop and enlarged Arabic/mobile choices remain readable.','Day, dusk, night and weather reviewed. Windowless halls remain sheltered; job-fair precipitation stays in background.','Actual results, simulation RNG, receipts, supplies, cash, vehicle, duplicate selection, day boundary, old save recovery and offline reload tested.','Supplied client checking choice yielded sanity 5 to 6 and credibility 5 to 7, unchanged morale/supplies and five remaining travelers.','Hearing failure, secured victory, exhaustion and narrow presentation regressions passed on this candidate.'],'remaining_findings':['Most narrative families and derived cast coverage remain unfinished.','Final combined release/update, route, accessibility and performance acceptance remains; candidate is about 203 MB.','All 36 proposed encounter-mechanics records remain explicit dependencies.'],'catalog_counts':dict(counts)}
(review/'road-c06-review.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n')
corrections_path=review/'road-c06-corrections.json'
corrections=json.loads(corrections_path.read_text());corrections['status']='Source corrections and representative runtime compositions inspected; integrated and verified.'
corrections_path.write_text(json.dumps(corrections,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'catalog_records':len(current),'counts':dict(counts),'source_audit_images':len(audit)}))
