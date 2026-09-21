"""Add test-crew entry pages to a disposable local build, outside shipped sources."""
from pathlib import Path
import json
import sys

root = Path(__file__).resolve().parent
build = Path(sys.argv[1])
for filename, save, route in [
    ('hearing-review.html', 'hearing-review-save.json', '/play/boss'),
    ('before-hearing-review.html', 'before-hearing-save.json', '/play/travel'),
]:
    template = 'before-hearing-verification/game-client/checkpoint.json' if route == '/play/travel' else 'skill-client/checkpoint.json'
    checkpoint = json.loads((root / template).read_text())
    checkpoint['state'] = json.loads((root / save).read_text())
    # Store JSON as text: parsing it in JavaScript would round a 64-bit RNG seed.
    raw_checkpoint = json.dumps(json.dumps(checkpoint)).replace('</', '<\\/')
    page = '''<!doctype html><meta charset="utf-8"><title>Hearing review</title>
<p>Opening the review test crew…</p><script>
const key='dystrail.autosave.v1';
if(localStorage.getItem(key)&&!localStorage.getItem('dystrail.hearing-review.previous'))localStorage.setItem('dystrail.hearing-review.previous',localStorage.getItem(key));
localStorage.setItem(key,CHECKPOINT);location.replace(ROUTE);
</script>'''.replace('CHECKPOINT', raw_checkpoint).replace('ROUTE', json.dumps(route))
    (build / filename).write_text(page)
