"""Record completed checks for this production slice without closing the full goal."""
import json
from pathlib import Path
root=Path(__file__).resolve().parents[2]
review=root/'review/art-satire'
assert '28 passed' in (review/'barter-hearing-regression.log').read_text()
assert '6 passed' in (review/'barter-boundary-browser.log').read_text()
assert '97 passed; 0 failed' in (review/'barter-final-native.log').read_text()
offline=json.loads((review/'barter-offline-integrity.json').read_text())
record={
 'date':'2026-09-14', 'scope':'Three existing community exchange families, nine A/B/C units, eighteen offered/completed frames',
 'status':'integrated and rendered review completed; overall visual-world goal remains active',
 'current_build':offline['revision'],
 'artwork':{'atlases':['barter-a','barter-b','barter-c'],'method':'Built-in image generation; original images retained without raster modification',
   'staging':'Same resident, table and goods before/after. Goods exchange positions. No repairs, new crew, writing or currency depicted.',
   'rendering':'Explicit SVG crop bounds, clip paths and 9:4 scene regions preserve full frames. Actual time/weather and surviving crew supplied separately.'},
 'mechanics':'Existing atomic trades, one exchange per stop, thirty-minute cost, affordability and capacity limits retained; no engine edits in this slice.',
 'persistence':'Offers sealed on opening with route/stop/family identity; frozen after successful exchange. Save/import and recovery/offline preserve inventory and shared stop restriction. Simulation RNG is not used for art selection.',
 'locales':{'complete':['en','es','it','ar'],'other_supported_locales':'Explicit English fallback'},
 'source_adaptations':['Food-assistance A uses the already verified CRS report shared with pantry work instead of the unavailable AP URL. Other variants reuse verified source records with their qualifications.',
   'Feminine or neutral resident references; fixed workshop traveler counts are represented by the actual current crew, not baked into art.'],
 'validation':[
   {'build':'e7b43ea32d5cde037ebf','checks':['396 workspace tests; strict native and Wasm clippy','Desktop/phone scene matrix: 9 variants x 4 languages x 2 viewports, with offered/completed captures, actual receipts and offline recovery','28 combined barter, hearing and town-flow browser cases passed'],
    'evidence':['barter-workspace-tests.log','barter-clippy.log','barter-wasm-clippy.log','barter-first-pass/','barter-hearing-pass/'],
    'test_correction':'The first import assertion compared serialized HashSet tag order. It now compares sorted tags and exact spare quantities; the corrected import cases passed.'},
   {'build':offline['revision'],'checks':['97 web native tests and strict native/Wasm clippy after final duration/unavailable copy','Final desktop/phone scene matrix and availability/save checks','Six final boundary, newly sealed offer and hearing-rest recovery browser cases passed','All offline manifest lengths/SHA-256 and barter atlas source/build bytes verified'],
    'evidence':['barter-final-native.log','barter-final-clippy.log','barter-final-wasm-clippy.log','barter-final-pass/','barter-boundary-pass/','barter-offline-integrity.json'],
    'test_correction':'An initial boundary assertion expected minutes, but the existing localized duration helper displays 0.5 h. The corrected check uses the locale template; no game behavior was changed for the assertion.'}
 ],
 'counts_note':'Validation groups overlap; do not add them as unique test coverage.',
 'visual_review':['barter-first-pass/test-results/barter-ACT-BARTERBATTERY-A-en-completed-chromium.png','barter-first-pass/test-results/barter-ACT-BARTERTIRE-B-en-offer-chromium.png','barter-first-pass/test-results/barter-ACT-BARTERSUPPLIES-C-ar-completed-mobile.png','barter-client-final/shot-0.png'],
 'hearing':'All 60 captured source files still match the other checkout. Seven engine/tester files and 20 hearing locale subtrees remain preserved here; completed hearing behavior tested in the combined build.',
 'hearing_evidence':'barter-hearing-preservation.json',
 'remaining':'Rest variants, most other compatible scene/variant families, supporting and derived cast audits, and final whole-feature release/update/performance checks. All644 source records remain accounted for; the36 mechanics-dependent scenes remain inactive.',
 'deployment':'No commit or deployment in this slice.'
}
(review/'barter-review.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n')
path=review/'asset-catalog.json';catalog=json.loads(path.read_text())
for asset in catalog['assets']:
 if asset['id'].startswith('barter-'):asset['status']='integrated_render_reviewed'
for unit in catalog['units']:
 if unit['family'].startswith('ACT-BARTER'):unit['production_status']='integrated_render_reviewed'
path.write_text(json.dumps(catalog,ensure_ascii=False,indent=2)+'\n')
print('Recorded barter render, persistence, source and combined hearing verification.')
