"""Capture the current game with the unmodified, supplied web-game client.

Only disposable save fixtures are changed. Screenshots are not retouched.
"""
import concurrent.futures
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "review/art-satire/current-previews"
FIXTURES = ROOT / "review/art-satire/client-fixtures"
OUT.mkdir(exist_ok=True)
revision = json.loads(Path('/private/tmp/dystrail-art-satire-build/offline-manifest.json').read_text())['revision']

road = json.loads((FIXTURES / 'road.json').read_text())
road['state']['clock_minutes'] = 19 * 60
(OUT / 'road-dusk.json').write_text(json.dumps(road))
road['state']['clock_minutes'] = 13 * 60
road['travel_speed'] = 'Normal'
(OUT / 'road-day.json').write_text(json.dumps(road))

frames = [
    dict(id='cast', title='The revised cast', fixture=FIXTURES/'persona.json', status='Integrated', note='Six feminine or neutral character designs; supporting-cast coverage is still being completed.'),
    dict(id='road-dusk', title='Roadside at dusk', fixture=OUT/'road-dusk.json', status='Integrated', note='Clock lighting, surviving crew, editable billboard copy and bottom-left scene captions.'),
    dict(id='travel', title='The crew in motion', fixture=OUT/'road-day.json', click='button:text-is("Travel")', status='Integrated', note='Separate seats and driver pose, using the active crew roster.'),
    dict(id='announcement', title='A travel interruption', fixture=FIXTURES/'policy.json', status='Integrated', note='Graphical policy announcement; its ongoing effect remains in the status bar.'),
    dict(id='pantry', title='Satire and its consequence', fixture=FIXTURES/'pantry-A-donated.json', status='Integrated', note='A completed pantry action, its scene and actual resource changes.'),
    dict(id='town', title='A conversation over dinner', fixture=FIXTURES/'town-toledo-b.json', status='Partial', note='Toledo B preview: three meal poses work; the other three are unfinished. Automatic B/C selection remains disabled.'),
    dict(id='care', title='Care scene — unfinished staging', fixture=FIXTURES/'care-soda.json', status='Partial', note='Current satire and setting are bound together. The affected traveler and outcome-specific poses still need to be completed.'),
    dict(id='hearing', title='The hearing unfolds', fixture=FIXTURES/'hearing.json', click='button:text-is("Begin hearing")', status='Integrated', note='Consumes the incorporated hearing model and its committed round report; this is an actual resolved round.'),
]

def capture(task):
    frame, device, width = task
    dest = OUT/device/frame['id']
    dest.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, DYSTRAIL_CLIENT_FIXTURE=str(frame['fixture']), DYSTRAIL_CLIENT_WIDTH=str(width), DYSTRAIL_CLIENT_LANG='en', DYSTRAIL_CLIENT_FULL_PAGE='1')
    cmd = ['node','--experimental-loader','./review/art-satire/client-loader.mjs', '/Users/vanna/.codex/skills/develop-web-game/scripts/web_game_playwright_client.js', '--url','http://127.0.0.1:62531/play/','--actions-json','{"steps":[{"buttons":[],"frames":3}]}','--iterations','1','--pause-ms','3500','--screenshot-dir',str(dest)]
    if frame.get('click'):
        cmd += ['--click-selector', frame['click']]
    result = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, text=True)
    log = result.stdout + result.stderr
    (dest/'client.log').write_text(log)
    if result.returncode or 'Failed to click selector' in log or list(dest.glob('errors-*.json')):
        raise RuntimeError(f"Capture failed: {device}/{frame['id']}: {log[-1000:]}")
    state = json.loads((dest/'state-0.json').read_text())
    print(device, frame['id'], state.get('screen'), state.get('scene'), flush=True)
    return dict(device=device,frame=frame['id'],state=state)

tasks = [(frame,device,width) for frame in frames for device,width in [('desktop',1440),('mobile',390)]]
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    results = list(pool.map(capture,tasks))
payload = dict(revision=revision,frames=[{**f,'fixture':str(f['fixture'].relative_to(ROOT))} for f in frames],captures=results,full_feature_complete=False)
(OUT/'manifest.json').write_text(json.dumps(payload,ensure_ascii=False,indent=2)+'\n')
