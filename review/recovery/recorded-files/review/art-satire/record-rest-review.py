"""Record the finished rest slice and retain outstanding source refresh work."""
import collections,csv,hashlib,json,shutil
from pathlib import Path
root=Path(__file__).resolve().parents[2];review=root/'review/art-satire'
assert '26 passed' in (review/'rest-browser.log').read_text()
assert '2 passed' in (review/'rest-matrix-final.log').read_text()
assert '6 passed' in (review/'rest-corrected-browser.log').read_text()
assert '98 passed; 0 failed' in (review/'rest-final-native.log').read_text()
for name in ['rest-final-clippy.log','rest-final-wasm-clippy.log']:
    text=(review/name).read_text();assert 'Finished' in text and 'error:' not in text
offline=json.loads((review/'rest-offline-integrity.json').read_text())
dest=review/'rest-final-pass';dest.mkdir(exist_ok=True)
shutil.copytree(root/'dystrail-web/test-results',dest/'test-results',dirs_exist_ok=True)
shutil.copy2('/private/tmp/dystrail-art-satire-build/offline-manifest.json',dest/'offline-manifest.json')
record=dict(date='2026-09-14',status='rest variants integrated and visually reviewed; overall goal active',build=offline['revision'],
 units=['ACT-REST-A','ACT-REST-B','ACT-REST-C'],
 source='source-refresh-20260914/complete-pack.json; native document sync remains incomplete',
 art=dict(props='rest-props',clock_faces='rest-clock-faces',method='Built-in image generation/editing; no raster modification',
 staging='A/C show only active travelers beside the parked van before rest; B shows actual seated occupants. Completed rest closes curtains, turns the newspaper/phone and removes the offered pillow. The van keeps its dimensions and road position. Actual region, hour, minute and weather drive the scene.',
 alpha='Prop sheet has true alpha. The later clock-face edit returned RGB with a painted checkerboard; only the cream dials are consumed through tight ellipse clips. Surrounding pixels come from the original RGBA sheet.',
 previous='Earlier radio/keys props were superseded by the latest rest A/B source. Original images and prompt attempts retained.'),
 mechanics='Existing camp_rest unchanged: one day, recovery bounded by current values, up to one available supply consumed, two-day cooldown and ordinary daily settlement. No invented payment or food grant. Zero supplies remains permitted under the existing engine.',
 persistence='Day/family offer selection is sealed on opening. The accepted action and aftermath retain the original variant across next-day settlement, import and offline reload. Duplicate activation cannot apply a second rest.',
 locales=dict(complete=['en','es','it','ar'],other_offered_locales='Explicit English fallback'),
 adaptations=['B outcome says rest does what it can, since daily effects can offset recovery.','No food is drawn because supplies can be zero.','AP source links use accessible copies of the same reporting; Illinois uses the official statute.','No fixed crew count is baked into the artwork.'],
 validation=[
  '397 workspace tests passed before final presentation adjustments; 98 web library tests passed afterward.',
  'Strict native and Wasm clippy passed.',
  '26 combined rest/hearing browser cases passed on the first functional candidate.',
  'Final candidate: desktop/phone matrix covers all three variants in EN/ES/IT/AR, source links, active crew, stable van geometry, unobscured travelers, cooldowns and offline aftermath recovery.',
  'Six additional final-candidate cases passed: zero-supply/import/duplicate activation, actual clock/weather at morning/dusk/night, and hearing rest/manual-save recovery, on both viewports.',
  'All 149 packaged assets match offline-manifest lengths and integrity hashes; source/build rest atlas bytes match.',
  'Supplied game-client screenshot inspected after the road-anchor correction.'
 ],
 corrections=['Initial table scale overlapped one traveler and van scale differed between states; corrected.','Inherited direct-child road anchor compressed the van when combined with rest bottom positioning; corrected and visually verified.','A subsequent test compared viewport Y across page scrolling. It now checks layout dimensions and position relative to the scene; no gameplay change for that correction.','Initial native clippy rejected a test fixture field reassignment; fixture initialization corrected.'],
 evidence=['rest-native.log','rest-final-native.log','rest-final-clippy.log','rest-final-wasm-clippy.log','rest-browser.log','rest-first-pass/','rest-final-browser.log','rest-staging-failure/','rest-corrected-browser.log','rest-scroll-assertion/','rest-matrix-final.log','rest-final-pass/','rest-offline-integrity.json','rest-client-corrected/shot-0.png','rest-hearing-preservation.json'],
 hearing='Seven engine/tester files and twenty hearing locale subtrees remain byte/value identical to the completed feature snapshot; all sixty captured files remain unchanged in the other checkout. User handover makes combined delivery this task’s responsibility.',
 remaining='Eleven illustrated units require the newer satire revision; most other compatible scenes and variants, supporting/derived cast audit, final update/performance/accessibility and full release review remain. All 644 source records accounted for; 36 proposed-mechanics dependencies inactive.',deployment='No commit or deployment.')
(review/'rest-review.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n')
path=review/'asset-catalog.json';catalog=json.loads(path.read_text())
for asset in catalog['assets']:
    if asset['id'] in ['rest-props','rest-clock-faces']:asset['status']='integrated_render_reviewed'
for unit in catalog['units']:
    if unit['family']=='ACT-REST':unit['production_status']='integrated_render_reviewed'
assert sum(u['disposition']=='mechanics_dependency' for u in catalog['units'])==36
assert sum(u['production_status']=='source_refresh_required' for u in catalog['units'])==11
path.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
with (review/'source-refresh-20260914/coverage.csv').open('w',newline='') as f:
    writer=csv.writer(f);writer.writerow(['id','family','variant','runtime_key','disposition','production_status','asset','scene'])
    for u in catalog['units']:writer.writerow([u['id'],u['family'],u['variant'],u['runtime_key'],u['disposition'],u['production_status'],json.dumps(u.get('asset')),u.get('source_scene',{}).get('description','')])
print('Recorded rest completion, current source coverage and the remaining eleven illustrated-source refreshes.')
