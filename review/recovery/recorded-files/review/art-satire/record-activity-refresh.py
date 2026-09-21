"""Accept the eleven-source refresh only after corrected render checks pass."""
import collections,csv,hashlib,json,re,shutil
from pathlib import Path
root=Path(__file__).resolve().parents[2];r=root/'review/art-satire'
assert '30 passed' in (r/'barter-refresh-browser.log').read_text()
assert '6 passed' in (r/'work-refresh-browser.log').read_text()
assert '14 passed' in (r/'refresh-final-browser.log').read_text()
assert '2 passed' in (r/'refresh-label-browser.log').read_text()
assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed;', (r/'refresh-final-native.log').read_text())))==397
for file in ['refresh-label-clippy.log','refresh-label-wasm-clippy.log']:
    log=(r/file).read_text();assert 'Finished' in log and 'error:' not in log
manifest=json.loads((r/'refresh-final-offline-integrity.json').read_text());revision=manifest['revision']
ids=[u['id'] for u in json.loads((r/'source-refresh-20260914/illustrated-refresh-queue.json').read_text())]
p=r/'asset-catalog.json';catalog=json.loads(p.read_text())
for u in catalog['units']:
    if u['id'] in ids:u['production_status']='integrated_render_reviewed'
for a in catalog['assets']:
    if a['id']=='gather-refresh-20260914':a['status']='integrated_render_reviewed'
assert len(catalog['units'])==644
assert sum(u['disposition']=='mechanics_dependency' for u in catalog['units'])==36
assert all(u['production_status']!='source_refresh_required' for u in catalog['units'])
p.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
with (r/'source-refresh-20260914/coverage.csv').open('w',newline='') as f:
    writer=csv.writer(f);writer.writerow(['id','family','variant','runtime_key','disposition','production_status','asset','scene'])
    for u in catalog['units']:writer.writerow([u['id'],u['family'],u['variant'],u['runtime_key'],u['disposition'],u['production_status'],json.dumps(u.get('asset')),u.get('source_scene',{}).get('description','')])
dest=r/'refresh-label-pass';dest.mkdir(exist_ok=True);shutil.copytree(root/'dystrail-web/test-results',dest/'test-results',dirs_exist_ok=True)
shutil.copy2('/private/tmp/dystrail-art-satire-build/offline-manifest.json',dest/'offline-manifest.json')
record=dict(build=revision,status='eleven earlier illustrated records refreshed to current satire and visually reviewed; full goal active',units=ids,source='source-refresh-20260914/complete-pack.json, verified against the user-designated Satire Workshop handoff',assets=['barter-refresh-20260914','work-refresh-20260914','gather-refresh-20260914'],locales=['en','es','it','ar'],fallback='English narrative and prop label in other offered locales',staging=['Barter: closed provisions bags and intact tires swap positions; pay slip/receipt stay with resident.','Work: sorting/cleaning/teardown appears only after the action; rewards match engine receipts.','Gather: empty baskets fill; tractor stays parked and unrepaired; reused wooden crate stays empty; corn/baseball comparison stays with farmer.'],adaptations='Only two English farmer pronouns changed from he/his to she/her for the user’s cast brief; translations use feminine residents. Exact differences in refresh-copy-adaptations.json.',label='Crate stamp is native localized SVG lettering on the blank prop, with matching screen-reader text. Spanish and Italian retain cap height with an explicit width to fit the painted label.',validation=['397 workspace tests passed before the final label-width adjustment.','Strict native and Wasm clippy passed after the label adjustment.','30 barter/hearing cases passed at the barter checkpoint.','Six complete activity cases passed at the work checkpoint.','Fourteen combined activity/barter/hearing-rest cases passed; two activity matrices exposed oversized Spanish lettering.','After correction, both desktop/phone activity matrices passed across all twelve variants in EN/ES/IT/AR; runtime SVG bounds check verifies label fit.','Actual receipts, same RNG, absent-crew filtering, source links, offline recovery, imports, duplicate activation, cooldown and day-boundary behavior verified.','All 152 offline assets match manifest byte lengths and integrity hashes; all three source atlas files match packaged bytes.','Seven hearing engine/tester files, twenty hearing locale subtrees and sixty source snapshot files preserved.'],evidence=['barter-refresh-review.json','work-refresh-review.json','refresh-label-pass/','refresh-pre-label-fix/','refresh-final-browser.log','refresh-label-browser.log','refresh-final-native.log','refresh-label-clippy.log','refresh-label-wasm-clippy.log','refresh-final-offline-integrity.json','refresh-final-hearing-preservation.json','gather-refresh-client/'],remaining='Most road/town/care/ally/repair/condition/departure/hearing/ending scene variants, supporting and derived cast audits, and full release/update/performance/accessibility acceptance remain. 36 proposed-mechanics dependencies remain inactive.',deployment='No commit or deployment.')
(r/'activity-refresh-review.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n')
print(revision,collections.Counter(u['production_status'] for u in catalog['units']))
